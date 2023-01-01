use im_rc::Vector;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use types::EvalError;
use types::EvalError::WrongArgAmount;
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

    pub fn new_env(&self) -> Self {
        Env {
            0: Rc::new(InnerEnv {
                data: RefCell::new(HashMap::new()),
                outer: Some(self.clone()),
            }),
        }
    }
    pub fn bind2(&self, symbols: MalType, values: MalType) -> Result<(), EvalError> {
        let symbols = symbols.to_list()?;
        let values = values.to_list()?;
        self.bind(symbols, values)
    }

    pub fn bind(
        &self,
        symbols: Vector<MalType>,
        mut values: Vector<MalType>,
    ) -> Result<(), EvalError> {
        let mut symbols = symbols.into_iter();
        loop {
            if let Some(symbol) = symbols.next() {
                let symbol = symbol.to_symbol()?;
                if symbol == "&" {
                    let symbol = symbols.next().ok_or(WrongArgAmount)?.to_symbol()?;
                    self.set(symbol, MalType::List(values));
                    break;
                } else {
                    let value = values.pop_front().ok_or(WrongArgAmount)?.clone();
                    self.set(symbol, value);
                }
            } else {
                break;
            }
        }
        Ok(())
    }
    pub fn new_bind(
        &self,
        symbols: Vector<MalType>,
        values: Vector<MalType>,
    ) -> Result<Self, EvalError> {
        let new_env = self.new_env();
        new_env.bind(symbols, values)?;
        Ok(new_env)
    }
}
