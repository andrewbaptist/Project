use iced::{
    widget::{
        button, canvas, column, container, pane_grid, pane_grid::Configuration, pick_list, row,
        text, text_input, Container,
    },
    Fill,
};
use serialport::SerialPortInfo;
mod graph;
mod port;
mod style;
use graph::graph::FloatingGraph;
enum Pane {
    Graph(FloatingGraph),
    Controls,
}
#[derive(Debug, Clone)]
enum Message {
    Resize(pane_grid::ResizeEvent),
    Move(pane_grid::DragEvent),
    Save,
    PathChanged(String),
    ChangePort(String),
    Split(pane_grid::Pane),
}
struct App {
    panes: pane_grid::State<Pane>,
    path: String,
    ports: Result<Vec<SerialPortInfo>, serialport::Error>,
    port: Option<String>,
}
impl Default for App {
    fn default() -> App {
        App::new()
    }
}
impl App {
    fn new() -> Self {
        let config = Configuration::Pane(Pane::Controls);
        let g_state = pane_grid::State::with_configuration(config);
        App {
            panes: g_state,
            path: "graph1.csv".to_string(),
            ports: serialport::available_ports(),
            port: None,
        }
    }
    fn view(&self) -> Container<Message> {
        let grid = pane_grid(&self.panes, |pane, state, _minimized| {
            let title_text: String;
            pane_grid::Content::<Message>::new(match state {
                Pane::Graph(g) => {
                    title_text = "Graph".to_string();
                    graph_pane(g)
                }
                Pane::Controls => {
                    title_text = "App Controls".to_string();
                    controls_pane(&self.ports, self.port.clone(), self.path.clone(), pane)
                }
            })
            .title_bar(
                pane_grid::TitleBar::new(container(text(title_text)))
                    .style(style::style::title)
                    .padding(5),
            )
        })
        .spacing(10)
        .on_resize(10, Message::Resize)
        .on_drag(Message::Move);
        container(grid).style(style::style::app_s).padding(10)
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::Resize(e) => self.panes.resize(e.split, e.ratio),
            Message::Move(pane_grid::DragEvent::Dropped { pane, target }) => {
                self.panes.drop(pane, target)
            }
            Message::Move(_) => {}
            Message::Save => write_file(
                self.panes
                    .iter()
                    .filter_map(|(_p, t)| match t {
                        Pane::Graph(g) => Some(&g.values),
                        _ => None,
                    })
                    .collect(),
                &self.path,
            ),
            Message::PathChanged(path) => self.path = path,
            Message::ChangePort(port) => self.port = Some(port),
            Message::Split(pane) => {
                self.panes.split(
                    pane_grid::Axis::Horizontal,
                    pane,
                    Pane::Graph(FloatingGraph::new(
                        function(1000),
                        0.0,
                        0.0,
                        self.path.parse().ok(),
                    )),
                );
            }
        }
        let _: Vec<_> = self
            .panes
            .iter_mut()
            .map(|(_, t)| match t {
                Pane::Graph(g) => g.update(),
                _ => None,
            })
            .collect();
    }
}
fn controls_pane(
    ports: &Result<Vec<SerialPortInfo>, serialport::Error>,
    current_port: Option<String>,
    path: String,
    pane: pane_grid::Pane,
) -> Container<Message> {
    container(
        column![
            pick_list(
                match ports {
                    Ok(ports) => ports.iter().map(|port| port.port_name.clone()).collect(),
                    Err(err) => vec![err.to_string()],
                },
                current_port,
                Message::ChangePort
            )
            .placeholder("Select a Port"),
            button(row![
                text("save to:"),
                text_input("Path", &path).on_input(Message::PathChanged)
            ])
            .width(Fill)
            .on_press(Message::Save)
            .padding(15),
            button("New Graph").on_press(Message::Split(pane))
        ]
        .spacing(10),
    )
    .style(style::style::graph)
    .width(Fill)
    .height(Fill)
    .padding(10)
}
fn graph_pane(graph: &FloatingGraph) -> Container<Message> {
    container(column![canvas(graph).width(Fill).height(Fill),])
        .padding(10)
        .style(style::style::graph)
}
fn function(x_size: usize) -> Vec<f32> {
    (0..x_size)
        .map(|x| ((x as f32 * 0.01).sin() + 1.0))
        .collect()
}
fn write_file(_data: Vec<&Vec<f32>>, _path: &String) {
    todo!()
}
fn main() {
    let _ = iced::application("Graph", App::update, App::view).run();
}
