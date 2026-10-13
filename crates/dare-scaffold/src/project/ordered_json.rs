//! Insertion-ordered JSON value, used to rewrite `package.json` the way
//! `fs.readJson` + `fs.writeJson` do (key order preserved).
//!
//! Enabling serde_json's `preserve_order` feature would change `Map` for the
//! whole workspace (feature unification), so keep ordering local instead.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum OrderedJson {
    Scalar(Value),
    Array(Vec<OrderedJson>),
    Object(Vec<(String, OrderedJson)>),
}

impl OrderedJson {
    /// Insert or replace `key` in an object, keeping its position.
    pub fn set(&mut self, key: &str, value: OrderedJson) -> bool {
        let OrderedJson::Object(entries) = self else {
            return false;
        };
        match entries.iter_mut().find(|(k, _)| k == key) {
            Some((_, v)) => *v = value,
            None => entries.push((key.to_string(), value)),
        }
        true
    }
}

/// Object with keys in the given order.
pub fn obj(entries: Vec<(&str, OrderedJson)>) -> OrderedJson {
    OrderedJson::Object(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

pub fn arr(items: Vec<OrderedJson>) -> OrderedJson {
    OrderedJson::Array(items)
}

pub fn s(v: &str) -> OrderedJson {
    OrderedJson::Scalar(Value::String(v.to_string()))
}

pub fn b(v: bool) -> OrderedJson {
    OrderedJson::Scalar(Value::Bool(v))
}

pub fn int(v: i64) -> OrderedJson {
    OrderedJson::Scalar(Value::from(v))
}

pub fn num(v: f64) -> OrderedJson {
    OrderedJson::Scalar(Value::from(v))
}

pub fn null() -> OrderedJson {
    OrderedJson::Scalar(Value::Null)
}

impl Serialize for OrderedJson {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            OrderedJson::Scalar(v) => v.serialize(s),
            OrderedJson::Array(items) => {
                let mut seq = s.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            OrderedJson::Object(entries) => {
                let mut map = s.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

struct OrderedVisitor;

impl<'de> Visitor<'de> for OrderedVisitor {
    type Value = OrderedJson;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::Bool(v)))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::from(v)))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::from(v)))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::from(v)))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::String(v.to_string())))
    }
    fn visit_string<E: de::Error>(self, v: String) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::String(v)))
    }
    fn visit_unit<E: de::Error>(self) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::Null))
    }
    fn visit_none<E: de::Error>(self) -> Result<OrderedJson, E> {
        Ok(OrderedJson::Scalar(Value::Null))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<OrderedJson, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(OrderedJson::Array(items))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<OrderedJson, A::Error> {
        let mut entries: Vec<(String, OrderedJson)> = Vec::new();
        while let Some((k, v)) = map.next_entry::<String, OrderedJson>()? {
            // JSON.parse keeps the last duplicate at the first key's position.
            match entries.iter_mut().find(|(ek, _)| *ek == k) {
                Some((_, ev)) => *ev = v,
                None => entries.push((k, v)),
            }
        }
        Ok(OrderedJson::Object(entries))
    }
}

impl<'de> Deserialize<'de> for OrderedJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(OrderedVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_key_order() {
        let src = "{\n  \"name\": \"vite-project\",\n  \"scripts\": {\n    \"dev\": \"vite\",\n    \"build\": \"vue-tsc -b && vite build\"\n  },\n  \"private\": true\n}";
        let mut v: OrderedJson = serde_json::from_str(src).expect("parse");
        assert!(v.set("name", OrderedJson::Scalar(Value::String("app".into()))));
        let out = serde_json::to_string_pretty(&v).expect("ser");
        assert_eq!(out, src.replace("vite-project", "app"));
    }
}
