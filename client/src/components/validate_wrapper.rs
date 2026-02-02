use crate::utils::validator::Validator;
use crate::utils::validator::Validators;
use crate::SharedString;
use leptos::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ValidateData<T: Clone, S: Storage<RwSignal<T>> = SyncStorage> {
    data: RwSignal<T, S>,
    validators: Arc<Validators<T>>,
    error: RwSignal<Option<SharedString>>,
}

impl<T> Default for ValidateData<T, SyncStorage>
where
    T: Clone + Default + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new(Default::default(), None)
    }
}

impl<T> From<T> for ValidateData<T, SyncStorage>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Self::new(value, None)
    }
}

impl<T> Default for ValidateData<T, LocalStorage>
where
    T: Clone + Default + 'static,
{
    fn default() -> Self {
        Self::new_local(Default::default(), None)
    }
}

impl<T> From<T> for ValidateData<T, LocalStorage>
where
    T: Clone + 'static,
{
    fn from(value: T) -> Self {
        Self::new_local(value, None)
    }
}

impl<T> ValidateData<T, SyncStorage>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(data: T, validators: Option<Validators<T>>) -> Self {
        Self {
            data: RwSignal::new(data),
            validators: Arc::new(validators.unwrap_or_default()),
            error: RwSignal::new(None),
        }
    }
}

impl<T: Clone + 'static> ValidateData<T, LocalStorage> {
    pub fn new_local(data: T, validators: Option<Validators<T>>) -> Self {
        Self {
            data: RwSignal::new_local(data),
            validators: Arc::new(validators.unwrap_or_default()),
            error: RwSignal::new(None),
        }
    }
}

impl<T, S> ValidateData<T, S>
where
    T: Clone + 'static,
    S: Storage<RwSignal<T>>,
{
    pub fn from_state(
        data_state: RwSignal<T, S>,
        error_state: Option<RwSignal<Option<SharedString>>>,
        validators: Option<Validators<T>>,
    ) -> Self {
        Self {
            data: data_state,
            validators: Arc::new(validators.unwrap_or_default()),
            error: error_state.unwrap_or_default(),
        }
    }
    pub fn data(&self) -> RwSignal<T, S> {
        self.data.clone()
    }
    pub fn validators(&self) -> Arc<Validators<T>> {
        self.validators.clone()
    }
    pub fn error(&self) -> RwSignal<Option<SharedString>> {
        self.error.clone()
    }
    // pub fn get_state(&self) -> Option<RwSignal<T>> {
    //     return self.data.get_state();
    // }
    pub fn get(&self) -> T
    where
        RwSignal<T, S>: Get<Value = T>,
    {
        return self.data.get();
    }
    pub fn set(&self, data: T)
    where
        RwSignal<T, S>: Set<Value = T>,
    {
        return self.data.set(data);
    }
    pub fn set_error(&self, error: SharedString) {
        self.error.set(Some(error));
    }
    pub fn clear_error(&self) {
        self.error.set(None);
    }

    pub fn validate(&self, update_view: bool) -> Result<(), SharedString>
    where
        RwSignal<T, S>: Get<Value = T>,
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

    pub fn listener(&self) -> UnsyncCallback<T>
    where
        S: Clone + 'static,
        RwSignal<T, S>: Get<Value = T>,
    {
        let clone = self.clone();
        UnsyncCallback::new(move |_: T| {
            clone.validate(true);
        })
    }

    // pub fn view(&self, renderer: impl ValidateDataRenderer<T> + 'static) -> impl IntoView
    // where
    //     T: Clone + PartialEq + 'static,
    // {
    //     self.view_with_style(renderer, None)
    // }
    // pub fn view_with_style(
    //     &self,
    //     renderer: impl ValidateDataRenderer<T> + 'static,
    //     style: Option<SharedString>,
    // ) -> impl IntoView
    // where
    //     T: Clone + PartialEq + 'static,
    // {
    //     let renderer = Arc::new(renderer);
    //     let data = self.data.clone();
    //     let validators = self.validators.clone();
    //     self.error
    //         .view(move |error: RwSignal<Option<SharedString>>| {
    //             let renderer = renderer.clone();
    //             let validators = validators.clone();
    //             view! {
    //                 <ValidateWrapper error={error.get()} style={style.clone()}>
    //                     {
    //                         data.view(move |data: RwSignal<T>| {
    //                             let validators = validators.clone();
    //                             let error = error.clone();
    //                             renderer.render(data, UnsyncCallback::new(move |value| {
    //                                 validators.validate_into(&value, &error)
    //                             }))
    //                         })
    //                     }
    //                 </ValidateWrapper>
    //             }
    //         })
    // }
}

// pub trait ValidateDataRenderer<T> {
//     fn render(&self, value: RwSignal<T>, validator: UnsyncCallback<T>) -> AnyView;
// }

// impl<T, F> ValidateDataRenderer<T> for F
// where
//     T: 'static,
//     F: Fn(RwSignal<T>, UnsyncCallback<T>) -> AnyView,
// {
//     fn render(&self, value: RwSignal<T>, validator: UnsyncCallback<T>) -> AnyView {
//         self(value, validator)
//     }
// }

#[component]
pub fn ValidateWrapper(
    error: RwSignal<Option<SharedString>>,
    #[prop(into, optional)] style: SharedString,
    children: Children,
) -> impl IntoView {
    let text = {
        let error = error.clone();
        move || error.get().unwrap_or_else(|| SharedString::from("　"))
    };
    let text_style = {
        let error = error.clone();
        move || {
            if error.read().is_some() {
                "margin: 0;padding-bottom: 0.25em;color:red;"
            } else {
                "margin: 0;padding-bottom: 0.25em;"
            }
        }
    };
    view! {
        <div style={style}>
            { children() }
            <p style={text_style}>{text}</p>
        </div>
    }
}
