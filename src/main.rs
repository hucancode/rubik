use rubik::App;
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    env_logger::init();
    let mut app = App::default();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}
