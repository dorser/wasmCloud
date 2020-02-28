#[macro_use]
extern crate wascc_codec as codec;

#[macro_use]
extern crate log;

use prost::Message;
use ::redis_streams::{
    Client, Connection, ErrorKind, RedisError, RedisResult, StreamCommands, Value,
};

use codec::capabilities::{CapabilityProvider, Dispatcher, NullDispatcher};
use codec::core::OP_CONFIGURE;
use codec::eventstreams::{self, Event, StreamQuery, StreamResults, WriteResponse};
use wascc_codec::core::CapabilityConfiguration;

use std::error::Error;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

capability_provider!(RedisStreamsProvider, RedisStreamsProvider::new);

const CAPABILITY_ID: &str = "wascc:eventstreams";

pub struct RedisStreamsProvider {
    dispatcher: RwLock<Box<dyn Dispatcher>>,
    clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl Default for RedisStreamsProvider {
    fn default() -> Self {
        match env_logger::try_init() {
            Ok(_) => {}
            Err(_) => {}
        };

        RedisStreamsProvider {
            dispatcher: RwLock::new(Box::new(NullDispatcher::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl RedisStreamsProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn configure(
        &self,
        config: impl Into<CapabilityConfiguration>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let config = config.into();
        let c = initialize_client(config.clone())?;

        self.clients.write().unwrap().insert(config.module, c);
        Ok(vec![])
    }

    fn actor_con(&self, actor: &str) -> RedisResult<Connection> {
        let lock = self.clients.read().unwrap();
        if let Some(client) = lock.get(actor) {
            client.get_connection()
        } else {
            Err(RedisError::from((
                ErrorKind::InvalidClientConfig,
                "No client for this actor. Did the host configure it?",
            )))
        }
    }

    fn write_event(&self, actor: &str, event: Event) -> Result<Vec<u8>, Box<dyn Error>> {
        let data = map_to_tuples(event.values);
        let res: String = self.actor_con(actor)?.xadd(event.stream, "*", &data)?;
        Ok(bytes(WriteResponse { event_id: res }))
    }

    fn query_stream(&self, actor: &str, query: StreamQuery) -> Result<Vec<u8>, Box<dyn Error>> {
        let sid = query.stream_id.to_string();
        let items = if let Some(time_range) = query.range {
            if query.count > 0 {
                self.actor_con(actor)?.xrange_count(
                    query.stream_id,
                    time_range.min_time,
                    time_range.max_time,
                    query.count,
                )?
            } else {
                self.actor_con(actor)?.xrange(
                    query.stream_id,
                    time_range.min_time,
                    time_range.max_time,
                )?
            }
        } else {
            if query.count > 0 {
                self.actor_con(actor)?
                    .xrange_count(query.stream_id, "-", "+", query.count)?
            } else {
                self.actor_con(actor)?.xrange(query.stream_id, "-", "+")?
            }
        };
        let mut events = Vec::new();

        for stream_id in items.ids {
            let newmap = stream_id
                .map
                .iter()
                .map(|(k, v)| (k.to_string(), val_to_string(v)))
                .collect::<HashMap<String, String>>();
            events.push(Event {
                event_id: stream_id.id,
                stream: sid.to_string(),
                values: newmap,
            });
        }

        Ok(bytes(StreamResults { events }))
    }
}

impl CapabilityProvider for RedisStreamsProvider {
    fn capability_id(&self) -> &'static str {
        CAPABILITY_ID
    }

    // Invoked by the runtime host to give this provider plugin the ability to communicate
    // with actors
    fn configure_dispatch(&self, dispatcher: Box<dyn Dispatcher>) -> Result<(), Box<dyn Error>> {
        trace!("Dispatcher received.");
        let mut lock = self.dispatcher.write().unwrap();
        *lock = dispatcher;

        Ok(())
    }

    fn name(&self) -> &'static str {
        "waSCC Event Streams Provider (Redis)"
    }

    // Invoked by host runtime to allow an actor to make use of the capability
    // All providers MUST handle the "configure" message, even if no work will be done
    fn handle_call(&self, actor: &str, op: &str, msg: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        trace!("Received host call from {}, operation - {}", actor, op);

        match op {
            OP_CONFIGURE if actor == "system" => self.configure(msg.to_vec().as_ref()),
            eventstreams::OP_WRITE_EVENT => self.write_event(actor, Event::decode(msg).unwrap()),
            eventstreams::OP_QUERY_STREAM => {
                self.query_stream(actor, StreamQuery::decode(msg).unwrap())
            }
            _ => Err("bad dispatch".into()),
        }
    }
}

const ENV_REDIS_URL: &str = "URL";

fn initialize_client(config: CapabilityConfiguration) -> Result<Client, Box<dyn Error>> {
    let redis_url = match config.values.get(ENV_REDIS_URL) {
        Some(v) => v,
        None => "redis://0.0.0.0:6379/",
    }
    .to_string();

    info!(
        "Attempting to connect {} to Redis at {}",
        config.module, redis_url
    );
    match Client::open(redis_url.as_ref()) {
        Ok(c) => Ok(c),
        Err(e) => Err(format!("Failed to connect to redis: {}", e).into()),
    }
}

fn map_to_tuples(map: HashMap<String, String>) -> Vec<(String, String)> {
    map.into_iter().collect()
}

fn bytes(msg: impl prost::Message) -> Vec<u8> {
    let mut buf = Vec::new();
    msg.encode(&mut buf).unwrap();
    buf
}

// Extracts Redis arbitrary binary data as a string
fn val_to_string(val: &Value) -> String {
    if let Value::Data(vec) = val {
        ::std::str::from_utf8(&vec).unwrap().to_string()
    } else {
        "??".to_string()
    }
}


#[cfg(test)]
mod test {    
    use super::*;
    use std::collections::HashMap;
    use redis_streams::Commands;
    // **==- REQUIRES A RUNNING REDIS INSTANCE ON LOCALHOST -==**

    #[test]
    fn round_trip() {                
        let prov = RedisStreamsProvider::new();
        let config = CapabilityConfiguration {
            module: "testing-actor".to_string(),
            values: gen_config(),
        };

        let c = initialize_client(config.clone()).unwrap(); 
        let _res: bool = c.get_connection().unwrap().del("my-stream").unwrap(); // make sure we start with an empty stream
        prov.configure(config).unwrap();

        for _ in 0..6 {
            let ev = Event{
                event_id: "".to_string(),
                stream: "my-stream".to_string(),
                values: gen_values(),
            };
            let mut buf = Vec::new();
            ev.encode(&mut buf).unwrap();
            let _res = prov.handle_call("testing-actor", eventstreams::OP_WRITE_EVENT, &buf).unwrap();            
        }

        let mut buf = Vec::new();
        let query = StreamQuery{
            count: 0,
            range: None,
            stream_id: "my-stream".to_string(),
        };
        query.encode(&mut buf).unwrap();
        let res = prov.handle_call("testing-actor", eventstreams::OP_QUERY_STREAM, &buf).unwrap();
        let query_res = StreamResults::decode(res.as_ref()).unwrap();
        assert_eq!(6, query_res.events.len());
        assert_eq!(query_res.events[0].values["scruffy-looking"], "nerf-herder");
        let _res: bool = c.get_connection().unwrap().del("my-stream").unwrap(); // make sure we start with an empty stream
    }

    fn gen_config() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("URL".to_string(), "redis://0.0.0.0:6379/".to_string());
        h
    }

    fn gen_values() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("test".to_string(), "ok".to_string());
        h.insert("scruffy-looking".to_string(), "nerf-herder".to_string());
        h
    }
}