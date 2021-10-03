use log::{debug, error, info};
use muicss_yew::{
    button::{Button, Color, Variant},
    container::Container,
    input::{Input, InputType},
    textarea::Textarea,
};
use std::collections::HashMap;
use yew::{
    events,
    format::Json,
    html,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
    Classes, Component, ComponentLink, Html, ShouldRender,
};

enum Msg {
    Url(String),
    Submit,
    ServerResponse(Result<String, anyhow::Error>),
}

struct Model {
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    url: String,
    short_url: String,
    cache: HashMap<String, String>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            fetch_task: None,
            url: String::new(),
            short_url: String::new(),
            cache: HashMap::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Url(url) => {
                self.url = url;
                false
            }
            Msg::Submit => {
                if let Some(short_url) = self.cache.get(&self.url) {
                    info!("Using cached short url for {} URL", self.url);
                    self.short_url = short_url.clone();

                    true
                } else {
                    debug!("Sending request to the short server");
                    let request = Request::get("http://asas")
                        .body(Ok(self.short_url.clone()))
                        .expect("I would be wondered if Request builder could fail");

                    let request_callback =
                        self.link
                            .batch_callback(|response: Response<Json<Result<String, anyhow::Error>>>| {
                                if response.status().is_success() {
                                    let Json(data) = response.into_body();
                                    vec![Msg::ServerResponse(data)]
                                } else {
                                    error!("Server responded with {}", response.status());
                                    Vec::new()
                                }
                            });

                    let task = FetchService::fetch(request, request_callback).expect("Failed to send a request");
                    self.fetch_task = Some(task);

                    false
                }
            }
            Msg::ServerResponse(data) => match data {
                Ok(data) => {
                    let short_url = data;
                    self.cache.insert(self.url.clone(), short_url);
                    true
                }
                Err(e) => {
                    error!("Failed to deserialize data from server -> {}", e);
                    false
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let onchange = self.link.batch_callback(|event: events::ChangeData| {
            if let events::ChangeData::Value(url) = event {
                vec![Msg::Url(url)]
            } else {
                vec![]
            }
        });

        let submit = self.link.callback(|_: events::MouseEvent| Msg::Submit);

        let button = || -> Classes {
            let mut button_class = Classes::new();
            button_class.push("mui-col-md-2");
            button_class.push("mui--pull-left");
            button_class
        };

        let grid = || -> Classes {
            let mut grid_class = Classes::new();
            grid_class.push("mui-row");
            grid_class
        };

        let input = || -> Classes {
            let mut input_class = Classes::new();
            input_class.push("mui--pull-left");
            input_class.push("mui-col-md-8");
            input_class
        };

        let textarea = || -> Classes {
            let mut textarea_class = Classes::new();
            textarea_class.push("mui-textfield");
            textarea_class.push("mui-col-md-8");
            textarea_class
        };

        html! {
            <>
                <Container class={grid()}>
                    <Input class={input()} input_type=InputType::Url floating_label=true onchange={onchange}> { "Url goes here" } </Input>
                    <Button class={button()} variant=Variant::Raised color=Color::Dark onclick=submit> { "Short it" } </Button>
                </Container>
                <h1 />
                <h1 />
                <Container class={grid()}>
                    <Textarea class={textarea()} value={self.short_url.clone()}>
                        {"Your short url: "}
                    </Textarea>
                </Container>
            </>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default().message_on_new_line());
    yew::start_app::<Model>();
}
