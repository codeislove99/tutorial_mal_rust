use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use MalType;

pub struct InnerEnv {
    data: RefCell<HashMap<String, MalType>>,
    outer: Option<Env>,
}

#[derive(Clone)]
pub struct Env(Rc<InnerEnv>);

impl Deref for Env {
    type Target = InnerEnv;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Env {
    pub fn new() -> Self {
        Env(Rc::new(InnerEnv {
            data: RefCell::new(HashMap::new()),
            outer: None,
        }))
    }
    pub fn set(&self, key: String, value: MalType) {
        self.data.borrow_mut().insert(key, value);
    }
    pub fn find(&self, key: &String) -> Option<Env> {
        if self.data.borrow().contains_key(key.as_str()) {
            Some(self.clone())
        } else {
            match &self.outer {
                Some(outer) => outer.find(key),
                None => None,
            }
        }
    }
    pub fn get(&self, key: &String) -> Option<MalType> {
        match self.find(key) {
            Some(env) => env.data.borrow().get(key.as_str()).cloned(),
            None => None,
        }
    }

    pub fn new_env(& self) -> Self{
        Env{
            0: Rc::new(InnerEnv {
                data: RefCell::new(HashMap::new()),
                outer: Some(self.clone())
            })
        }
    }
}
