//! A small native operation workbench for independently developed applications.
#![forbid(unsafe_code)]
use iced::{Element, Task, Theme, Color, Length, theme::Palette, widget::{button,column,row,scrollable,text,text_input,text_editor,pick_list,container}};
use localwright_core::{Error,ErrorKind,Handler,Job,Operation,Outcome,Request,Result};

/// Initial presets change client-area colors only, not historical widget geometry.
pub const THEMES: &[(&str,[u8;3],[u8;3],[u8;3])] = &[
 ("Modern Light",[250,250,250],[25,25,28],[29,90,155]),("Modern Dark",[30,31,36],[238,238,240],[125,180,255]),
 ("Graphite",[45,46,48],[245,245,245],[171,190,205]),("Midnight",[12,21,40],[235,240,255],[125,160,255]),
 ("Classic 95",[192,192,192],[0,0,0],[0,0,128]),("Classic 2000",[212,208,200],[0,0,0],[10,36,106]),
 ("Luna",[236,233,216],[25,25,25],[36,79,163]),("Silver",[224,226,234],[20,20,25],[70,85,130]),
 ("Aqua",[239,243,247],[20,35,50],[30,100,175]),("Workstation",[216,216,208],[20,20,20],[54,73,96]),
 ("Retro Beige",[241,231,205],[55,40,20],[120,70,30]),("Paper",[255,252,242],[35,32,29],[95,55,30]),
 ("Nordic",[46,52,64],[236,239,244],[136,192,208]),("Solarized Light",[253,246,227],[60,75,80],[0,91,137]),
 ("Solarized Dark",[0,43,54],[238,232,213],[95,185,220]),("Phosphor Green",[5,14,7],[130,255,145],[60,180,85]),
 ("Amber Terminal",[20,13,4],[255,200,95],[217,145,50]),("Monochrome",[240,240,240],[0,0,0],[75,75,75]),
 ("High Contrast Light",[255,255,255],[0,0,0],[0,0,140]),("High Contrast Dark",[0,0,0],[255,255,255],[255,230,0])];
fn rgb(c:[u8;3])->Color { Color::from_rgb8(c[0],c[1],c[2]) }
fn preset(name:&str)->Theme { let p=THEMES.iter().find(|p|p.0==name).unwrap_or(&THEMES[0]); Theme::custom(p.0.to_owned(),Palette{background:rgb(p.1),text:rgb(p.2),primary:rgb(p.3),success:Color::from_rgb8(30,140,80),danger:Color::from_rgb8(200,50,50)}) }
struct State { ops:&'static [Operation], handler:Handler, selected:String, input:String, output:String, parameter:String, editor:text_editor::Content, preview:String, theme:String, job:Option<Job>, status:String }
#[derive(Debug,Clone)] enum Message { Select(String), Input(String), Output(String), Parameter(String), Edit(text_editor::Action), Theme(String), Run, Cancel, Done(Result<Outcome>) }
impl State {
 fn update(&mut self,msg:Message)->Task<Message> {
  match msg {
   Message::Select(v)=>self.selected=v, Message::Input(v)=>self.input=v, Message::Output(v)=>self.output=v, Message::Parameter(v)=>self.parameter=v,
   Message::Edit(a)=>self.editor.perform(a), Message::Theme(v)=>self.theme=v,
   Message::Cancel=>if let Some(j)=&self.job {j.cancel();self.status="Cancellation requested; waiting for operation".into();},
   Message::Run=>{if self.job.is_some(){return Task::none();} let job=Job::new();self.job=Some(job.clone()); self.status=format!("Running job {}",job.id); self.preview.clear();
    let r=Request{operation:self.selected.clone(),input:self.input.clone().into(),output:self.output.clone().into(),parameter:self.parameter.clone(),draft:self.editor.text()}; let handler=self.handler;
    let (tx,rx)=futures::channel::oneshot::channel();
    let spawn=std::thread::Builder::new().name("localwright-job".into()).spawn(move||{let _=tx.send(handler(r,job));});
    if let Err(e)=spawn {self.job=None; self.status=e.to_string();return Task::none();}
    return Task::perform(async move {rx.await.unwrap_or_else(|_|Err(Error::new(ErrorKind::Internal,"Worker stopped unexpectedly")))},Message::Done);
   },
   Message::Done(result)=>{self.job=None;match result {Ok(o)=>{self.status=if let Some(p)=o.written {format!("Wrote {}",p.display())} else {"Finished".into()};self.preview=o.preview;for warning in o.warnings {self.preview.push_str(&format!("\nWarning: {warning}"));} if let Some(d)=o.draft {self.editor=text_editor::Content::with_text(&d);}},Err(e)=>{self.status=format!("Error: {:?}",e.kind);self.preview=e.message;}}}
  } Task::none()
 }
 fn view(&self)->Element<'_,Message> {
  let ops=self.ops.iter().map(|o|o.id.to_owned()).collect::<Vec<_>>();let themes=THEMES.iter().map(|t|t.0.to_owned()).collect::<Vec<_>>();
  let hint=self.ops.iter().find(|o|o.id==self.selected).map(|o|o.hint).unwrap_or("");
  let mut run=button("Run operation"); if self.job.is_none(){run=run.on_press(Message::Run);}
  let mut cancel=button("Cancel");if self.job.is_some(){cancel=cancel.on_press(Message::Cancel);}
  container(column![row![text("Operation"),pick_list(ops,Some(self.selected.clone()),Message::Select),text("Theme"),pick_list(themes,Some(self.theme.clone()),Message::Theme)].spacing(12),
    text(hint),text("Input path"),text_input("Path to local input",&self.input).on_input(Message::Input),text("New output path (never overwritten)"),text_input("Output path",&self.output).on_input(Message::Output),text("Operation parameter"),text_input("See operation description",&self.parameter).on_input(Message::Parameter),row![run,cancel,text(&self.status)].spacing(8),
    text("Editable draft / transcript"),{let editor=text_editor(&self.editor).height(Length::Fixed(160.0)); if self.job.is_none(){editor.on_action(Message::Edit)}else{editor}},text("Result and warnings"),scrollable(text(&self.preview)).height(Length::Fill)].spacing(8)).padding(16).into()
 }
}
/// Run a native window with one bounded background operation at a time.
pub fn run(title:&'static str,ops:&'static [Operation],handler:Handler)->iced::Result {
 iced::application(title,State::update,State::view).theme(|s:&State|preset(&s.theme)).run_with(move|| (State{ops,handler,selected:ops.first().map(|o|o.id).unwrap_or("").into(),input:String::new(),output:String::new(),parameter:String::new(),editor:text_editor::Content::new(),preview:String::new(),theme:"Modern Light".into(),job:None,status:"Ready".into()},Task::none()))
}
