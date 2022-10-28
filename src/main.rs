#[macro_use]
extern crate log;

use std::{collections::HashMap, fs::File, time::Duration};

use futures_channel::mpsc::UnboundedSender;
use futures_util::StreamExt;
use rand::{seq::SliceRandom, thread_rng, Rng};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::time::timeout;
use tokio_schedule::Job;
use tokio_tungstenite::{connect_async, tungstenite::Message};

struct IncomingMessage {
    msg: String,
    group: Option<u64>,
    sender: u64,
}

#[derive(Deserialize)]
struct Eat {
    #[serde(rename = "早餐")]
    breakfast: Vec<String>,
    #[serde(rename = "午餐")]
    lunch: Vec<String>,
    #[serde(rename = "晚餐")]
    dinner: Vec<String>,
    #[serde(rename = "夜宵")]
    midnight_snack: Vec<String>,
}

impl Eat {
    fn open() -> Option<Self> {
        if let Ok(file) = File::open("./data/eat.yml") {
            if let Ok(eat) = serde_yaml::from_reader::<_, Eat>(file) {
                return Some(eat);
            }
        }
        None
    }

    fn random_breakfast(&self) -> String {
        let mut rng = thread_rng();
        let def = String::from("西北风");
        format!("早餐：{}", self.breakfast.choose(&mut rng).unwrap_or(&def))
    }

    fn random_lunch(&self) -> String {
        let mut rng = thread_rng();
        let def = String::from("西北风");
        format!("午餐：{}", self.lunch.choose(&mut rng).unwrap_or(&def))
    }

    fn random_dinner(&self) -> String {
        let mut rng = thread_rng();
        let def = String::from("西北风");
        format!(
            "晚餐：{}",
            [&self.lunch[..], &self.dinner[..]]
                .concat()
                .choose(&mut rng)
                .unwrap_or(&def)
        )
    }

    fn random_midnight_snack(&self) -> String {
        let mut rng = thread_rng();
        let def = String::from("西北风");
        format!(
            "夜宵：{}",
            self.midnight_snack.choose(&mut rng).unwrap_or(&def)
        )
    }

    fn random(&self) -> String {
        let mut rng = thread_rng();
        let def = String::from("西北风");
        format!(
            "早餐：{}\n\
                午餐：{}\n\
                晚餐：{}\n\
                夜宵：{}",
            self.breakfast.choose(&mut rng).unwrap_or(&def),
            self.lunch.choose(&mut rng).unwrap_or(&def),
            [&self.lunch[..], &self.dinner[..]]
                .concat()
                .choose(&mut rng)
                .unwrap_or(&def),
            self.midnight_snack.choose(&mut rng).unwrap_or(&def),
        )
    }
}

#[derive(Serialize)]
struct Params<'a> {
    user_id: Option<u64>,
    group_id: Option<u64>,
    message: &'a str,
}

#[derive(Serialize)]
struct SendMessage<'a> {
    action: &'a str,
    params: Params<'a>,
}

/// FIXME: 换用类似路由的来实现
/// #example
///
/// ```
/// RouterBuilder::new(rx, tx)
///     .register("#help", |args| async {
///         "blahblah"
///     })
///     .register("#eat", "吃什么", |cmd, args| async {
///         // 返回 Stream<String>
///     })
///     .register(|message| async {
///         // 处理其他消息
///     })
///     .build()
///     .run()
///     .await
/// ```
async fn handle_message(incoming_message: IncomingMessage, tx: UnboundedSender<Message>) {
    let mut messages: Vec<String> = Vec::new();

    // FIXME: 换用正则来解析，避免出现诸如 index out of bounds 之类的问题
    if incoming_message.msg.starts_with("[CQ:json,data=") && incoming_message.msg.ends_with("]") {
        let text = &incoming_message.msg[14..incoming_message.msg.len() - 1];
        let json = html_escape::decode_html_entities(text);
        if let Ok(json) = serde_json::from_str::<HashMap<String, Value>>(&json) {
            if let Some(Value::Object(meta)) = json.get("meta") {
                if let Some(Value::Object(detail_1)) = meta.get("detail_1") {
                    if let (Some(Value::String(url)), Some(Value::String(title))) =
                        (detail_1.get("qqdocurl"), detail_1.get("desc"))
                    {
                        messages.push(format!(
                            "标题：{}\n\
                            链接：{}",
                            title, url
                        ));
                    }
                }
            }
        }
    }

    let lines = incoming_message.msg.lines();
    for line in lines {
        let mut ns = fasteval::EmptyNamespace;
        if let Ok(value) = fasteval::ez_eval(line, &mut ns) {
            if value.is_infinite() {
                messages.push(format!("[CQ:at,qq={}]太大了自己算去", incoming_message.sender));
            } else if value.trunc() == value {
                messages.push(format!("[CQ:at,qq={}]{}", incoming_message.sender, value as i128));
            } else {
                messages.push(format!("[CQ:at,qq={}]{}", incoming_message.sender, value));
            }
        }
    }


    let parts: Vec<&str> = incoming_message.msg.split_whitespace().collect();
    match parts[0] {
        "#help" | "#h" => {
            messages.push(
                "#help\n\
                    \u{20}\u{20}\u{20}\u{20}显示本帮助\n\
                    #random #r #roll\n\
                    \u{20}\u{20}\u{20}\u{20}掷骰子\n\
                    #cat 猫猫图\n\
                    \u{20}\u{20}\u{20}\u{20}猫猫图\n\
                    #dog 狗狗图\n\
                    \u{20}\u{20}\u{20}\u{20}狗狗图\n\
                    #eat 吃什么\n\
                    \u{20}\u{20}\u{20}\u{20}今天吃点什么呢……\n\
                    #breakfast 早上吃什么 早餐吃什么 早餐\n\
                    \u{20}\u{20}\u{20}\u{20}早上吃点什么呢……\n\
                    #lunch 中午吃什么 午餐吃什么 午餐\n\
                    \u{20}\u{20}\u{20}\u{20}中午吃点什么呢……\n\
                    #dinner 晚上吃什么 晚餐吃什么 晚餐\n\
                    \u{20}\u{20}\u{20}\u{20}晚上吃点什么呢……\n\
                    #midnight_snack 夜宵吃什么 宵夜吃什么 夜宵 宵夜\n\
                    \u{20}\u{20}\u{20}\u{20}夜宵吃点什么呢……\n\
                    #poem 念诗\n\
                    \u{20}\u{20}\u{20}\u{20}念句诗"
                    .into(),
            );
        }
        "#random" | "#r" | "#roll" => {
            let mut min: i32 = 0;
            let mut max: i32 = 100;
            if parts.len() == 2 {
                if let Ok(max_parsed) = parts[1].parse::<i32>() {
                    if max_parsed >= 10000 {
                        max = 10000
                    } else {
                        max = max_parsed;
                    }
                }
            } else if parts.len() > 2 {
                if let Ok(min_parsed) = parts[1].parse::<i32>() {
                    if min_parsed < -10000 {
                        min = -9999
                    } else {
                        min = min_parsed;
                    }
                }
                if let Ok(max_parsed) = parts[2].parse::<i32>() {
                    if max_parsed >= 10000 {
                        max = 10000
                    } else {
                        max = max_parsed;
                    }
                }
            }
            if max < min {
                max = min;
            }
            let mut rng = thread_rng();
            let dice = rng.gen_range(min..=max);
            messages.push(format!(
                "[CQ:at,qq={}]掷出了{}点！",
                incoming_message.sender, dice
            ));
        }
        "#cat" | "猫猫图" => {
            if let Ok(Ok(resp)) = timeout(
                Duration::from_secs(5),
                reqwest::get("https://api.thecatapi.com/v1/images/search"),
            )
            .await
            {
                if let Ok(resp) = resp.json::<Vec<Value>>().await {
                    if let Some(Value::Object(object)) = resp.first() {
                        if let Some(Value::String(url)) = object.get("url") {
                            messages.push(format!("[CQ:image,file={}]", url));
                        }
                    }
                }
            }
        }
        "#dog" | "狗狗图" => {
            if let Ok(Ok(resp)) = timeout(
                Duration::from_secs(5),
                reqwest::get("https://api.thedogapi.com/v1/images/search"),
            )
            .await
            {
                if let Ok(resp) = resp.json::<Vec<Value>>().await {
                    if let Some(Value::Object(object)) = resp.first() {
                        if let Some(Value::String(url)) = object.get("url") {
                            messages.push(format!("[CQ:image,file={}]", url));
                        }
                    }
                }
            }
        }
        "#sysinfo" => {
            let mut info = System::new_all();
            info.refresh_all();
            messages.push(format!(
                "系统：{}\n\
                内核版本：{}\n\
                系统版本：{}\n\
                内存用量：{} MiB / {} MiB\n\
                交换用量：{} MiB / {} MiB\n\
                核心负载：{}",
                info.name().unwrap_or_default(),
                info.kernel_version().unwrap_or_default(),
                info.os_version().unwrap_or_default(),
                info.used_memory() / 1048576,
                info.total_memory() / 1048576,
                info.used_swap() / 1048576,
                info.total_swap() / 1048576,
                info.cpus()
                    .iter()
                    .map(|cpu| format!("{:.2}%", cpu.cpu_usage()))
                    .collect::<Vec<String>>()
                    .join(" ")
            ));
        }
        "#eat" | "吃什么" => {
            if let Some(eat) = Eat::open() {
                messages.push(eat.random());
            }
        }
        "#breakfast" | "早上吃什么" | "早餐吃什么" | "早餐" => {
            if let Some(eat) = Eat::open() {
                messages.push(eat.random_breakfast());
            }
        }
        "#lunch" | "中午吃什么" | "午餐吃什么" | "午餐" => {
            if let Some(eat) = Eat::open() {
                messages.push(eat.random_lunch());
            }
        }
        "#dinner" | "晚上吃什么" | "晚餐吃什么" | "晚餐" => {
            if let Some(eat) = Eat::open() {
                messages.push(eat.random_dinner());
            }
        }
        "#midnight_snack" | "夜宵吃什么" | "宵夜吃什么" | "夜宵" | "宵夜" => {
            if let Some(eat) = Eat::open() {
                messages.push(eat.random_midnight_snack());
            }
        }
        "#poem" | "念诗" => {
            if let Ok(Ok(resp)) = timeout(
                Duration::from_secs(5),
                reqwest::get("https://v1.jinrishici.com/all.json"),
            )
            .await
            {
                if let Ok(resp) = resp.json::<HashMap<String, String>>().await {
                    if let Some(content) = resp.get("content") {
                        messages.push(content.to_string());
                    }
                }
            }
        }
        "#tarot" => {}
        _ => {}
    }

    for message in messages.iter() {
        let message = Some(SendMessage {
            action: "send_msg",
            params: Params {
                user_id: match incoming_message.group {
                    None => Some(incoming_message.sender),
                    Some(_) => None,
                },
                group_id: incoming_message.group,
                message: message.as_str(),
            },
        });

        let message = serde_json::to_string(&message).unwrap();
        if let Err(e) = tx.unbounded_send(Message::text(&message)) {
            warn!("未能发出消息 {}，错误信息 {}", &message, e.to_string());
        };
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "ws://127.0.0.1:54321";

    let ws_stream = connect_async(addr).await;
    if let Err(e) = ws_stream {
        warn!(
            "无法与来自 {} 的连接建立 WebSocket 通道，错误信息 {}",
            addr,
            e.to_string()
        );
        return;
    }

    let (ws_stream, _) = ws_stream.unwrap();
    let (sink, stream) = ws_stream.split();

    let (tx, rx) = futures_channel::mpsc::unbounded();

    let tx_cloned = tx.clone();

    tokio::spawn(
        tokio_schedule::every(1)
            .week()
            .on(chrono::Weekday::Sat)
            .at(21, 0, 0)
            .in_timezone(&chrono::Local)
            .perform(move || {
                let tx = tx_cloned.clone();

                async move {
                    let message = Some(SendMessage {
                        action: "send_msg",
                        params: Params {
                            user_id: None,
                            group_id: Some(790720353),
                            message:
                                "仙人仙彩开奖啦，请记得到金碟游乐场 (X: 8.6, Y: 5.7)处兑换奖励哦~",
                        },
                    });

                    let message = serde_json::to_string(&message).unwrap();
                    if let Err(e) = tx.clone().unbounded_send(Message::text(&message)) {
                        warn!("未能发出消息 {}，错误信息 {}", &message, e.to_string());
                    };
                }
            }),
    );

    let rx_to_ws = rx.map(Ok).forward(sink);
    let ws_to_tx = {
        stream.for_each(|msg| async {
            if let Ok(Message::Text(msg)) = msg {
                if let Ok(json_map) =
                    serde_json::from_str::<HashMap<String, serde_json::Value>>(&msg)
                {
                    if let (Some(Value::String(message)), Some(Value::Object(sender)), group_id) = (
                        json_map.get("message"),
                        json_map.get("sender"),
                        json_map.get("group_id"),
                    ) {
                        if let Some(Value::Number(sender)) = sender.get("user_id") {
                            if let Some(sender) = sender.as_u64() {
                                let mut incoming_message = IncomingMessage {
                                    msg: message.to_string(),
                                    sender: sender,
                                    group: None,
                                };
                                if let Some(serde_json::Value::Number(group_id)) = group_id {
                                    incoming_message.group = group_id.as_u64()
                                }

                                tokio::spawn(handle_message(incoming_message, tx.clone()));
                            }
                        }
                    }
                }
            }
        })
    };

    futures_util::pin_mut!(rx_to_ws, ws_to_tx);
    futures_util::future::select(rx_to_ws, ws_to_tx).await;
}
