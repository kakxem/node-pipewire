use std::{cell::RefCell, rc::Rc};

use pipewire::proxy::{Proxy, ProxyListener};

pub(crate) struct ProxyWrapper {
    internal: Rc<RefCell<ProxyInternal>>,
}

impl ProxyWrapper {
    pub fn new(proxy: Proxy) -> Self {
        return Self {
            internal: ProxyInternal::new(proxy),
        };
    }

    pub fn get_global_id(&self) -> u32 {
        return self.internal.borrow().global_id;
    }
}

struct ProxyInternal {
    pub proxy: Proxy,
    pub global_id: u32,
    pub listener: Option<ProxyListener>,
}

impl ProxyInternal {
    fn new(proxy: Proxy) -> Rc<RefCell<Self>> {
        let pxw = Rc::new(RefCell::new(Self {
            proxy: proxy,
            global_id: 0,
            listener: None,
        }));

        let listener = Some(
            pxw.borrow()
                .proxy
                .add_listener_local()
                .bound({
                    let pxw = pxw.clone();
                    move |id| {
                        let mut borrowed = pxw.borrow_mut();
                        borrowed.global_id = id;
                        borrowed.listener = None;
                    }
                })
                .register(),
        );

        pxw.borrow_mut().listener = listener;

        return pxw;
    }
}
