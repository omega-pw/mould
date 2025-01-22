use crate::utils::binding::Binding;
use crate::utils::validator::Validator;
use crate::utils::validator::Validators;
use crate::LightString;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct ValidateData<T: Clone> {
    pub data: Binding<T>,
    pub validators: Validators<T>,
    pub error: Binding<Option<AttrValue>>,
}

impl<T> Default for ValidateData<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self::new(Default::default(), None)
    }
}

impl<T: Clone> From<T> for ValidateData<T> {
    fn from(value: T) -> Self {
        Self::new(value, None)
    }
}

impl<T: Clone> ValidateData<T> {
    pub fn new(data: T, validators: Option<Validators<T>>) -> Self {
        Self {
            data: Binding::new(data),
            validators: validators.unwrap_or_default(),
            error: Binding::new(None),
        }
    }
    pub fn from_state(
        data_state: UseStateHandle<T>,
        error_state: Option<UseStateHandle<Option<LightString>>>,
        validators: Option<Validators<T>>,
    ) -> Self {
        Self {
            data: Binding::from(data_state),
            validators: validators.unwrap_or_default(),
            error: if let Some(error_state) = error_state {
                Binding::from(error_state)
            } else {
                Binding::new(None)
            },
        }
    }
    pub fn get_state(&self) -> Option<UseStateHandle<T>> {
        return self.data.get_state();
    }
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        return self.data.get();
    }
    pub fn set(&self, data: T)
    where
        T: Clone,
    {
        return self.data.set(data);
    }
    pub fn set_error(&self, error: AttrValue) {
        self.error.set(Some(error));
    }
    pub fn clear_error(&self) {
        self.error.set(None);
    }
    pub fn validate(&self, update_view: bool) -> Result<(), LightString>
    where
        T: Clone,
    {
        let error = self.validators.validate(&self.get());
        if update_view {
            self.error.set(error.clone());
        }
        if let Some(error) = error {
            Err(error)
        } else {
            Ok(())
        }
    }
    pub fn view(&self, renderer: impl ValidateDataRenderer<T> + 'static) -> Html
    where
        T: Clone + PartialEq + 'static,
    {
        self.view_with_style(renderer, None)
    }
    pub fn view_with_style(
        &self,
        renderer: impl ValidateDataRenderer<T> + 'static,
        style: Option<AttrValue>,
    ) -> Html
    where
        T: Clone + PartialEq + 'static,
    {
        let renderer = Arc::new(renderer);
        let data = self.data.clone();
        let validators = self.validators.clone();
        self.error
            .view(move |error: UseStateHandle<Option<AttrValue>>| {
                let renderer = renderer.clone();
                let validators = validators.clone();
                html! {
                    <ValidateWrapper error={error.deref().clone()} style={style.clone()}>
                        {
                            data.view(move |data: UseStateHandle<T>| {
                                let validators = validators.clone();
                                let error = error.clone();
                                renderer.render(data, Callback::from(move |value| {
                                    validators.validate_into(&value, &error)
                                }))
                            })
                        }
                    </ValidateWrapper>
                }
            })
    }
}

pub trait ValidateDataRenderer<T> {
    fn render(&self, value: UseStateHandle<T>, validator: Callback<T>) -> Html;
}

impl<T, F> ValidateDataRenderer<T> for F
where
    F: Fn(UseStateHandle<T>, Callback<T>) -> Html,
{
    fn render(&self, value: UseStateHandle<T>, validator: Callback<T>) -> Html {
        self(value, validator)
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub error: Option<AttrValue>,
    #[prop_or_default]
    pub style: Option<AttrValue>,
    pub children: Children,
}

#[function_component]
pub fn ValidateWrapper(props: &Props) -> Html {
    let (text, style) = if let Some(error) = props.error.as_ref() {
        (
            error.as_ref(),
            "margin: 0;padding-bottom: 0.25em;color:red;",
        )
    } else {
        ("　", "margin: 0;padding-bottom: 0.25em;")
    };
    html! {
        <div style={props.style.clone()}>
            { props.children.clone() }
            <p style={style}>{text}</p>
        </div>
    }
}
