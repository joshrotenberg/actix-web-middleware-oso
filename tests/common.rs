use oso::{Oso, PolarClass};

#[derive(Debug, PolarClass)]
pub struct User {
    #[polar(attribute)]
    pub name: String,
}

pub fn init_oso() -> Oso {
    let mut o = Oso::new();
    o.register_class(User::get_polar_class()).unwrap();
    o.load_str(r#"allow(actor, _action, _resource) if actor matches User{name: "alice"};"#)
        .unwrap();
    o
}
