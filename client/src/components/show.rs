use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub condition: bool,
    #[prop_or_default]
    pub class: Option<AttrValue>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub children: Children,
}

#[function_component]
pub fn Show(props: &Props) -> Html {
    let style = if let Some(style) = props.style.as_ref() {
        if props.condition {
            Some(style.clone())
        } else {
            Some(AttrValue::from(format!("{};display:none", style)))
        }
    } else {
        if props.condition {
            None
        } else {
            Some(AttrValue::from("display:none"))
        }
    };
    html! {
        <div class={props.class.clone()} style={style}>
            { props.children.clone() }
        </div>
    }
}
