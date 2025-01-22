use std::cell::Cell;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use yew::prelude::*;

fn get_cell_data<T>(cell: &Cell<T>) -> T
where
    T: Clone + Default,
{
    let value = cell.take();
    let value_clone = value.clone();
    cell.set(value_clone);
    return value;
}

#[derive(Clone)]
pub struct Binding<T> {
    inherit_state: bool,
    value: Arc<Cell<Option<T>>>,
    handle: Arc<Cell<Option<UseStateHandle<T>>>>,
}

impl<T> fmt::Debug for Binding<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(handle) = self.get_state() {
            write!(fmt, "{:?}", handle.deref())
        } else {
            write!(fmt, "{:?}", None::<T>)
        }
    }
}

impl<T> Binding<T> {
    pub fn new(value: T) -> Self {
        Self {
            inherit_state: false,
            value: Arc::new(Cell::new(Some(value))),
            handle: Arc::new(Cell::new(None)),
        }
    }
    pub fn init(&self, handle: UseStateHandle<T>) {
        self.handle.set(Some(handle));
    }
    pub fn init_callback(&self) -> Callback<UseStateHandle<T>>
    where
        T: 'static,
    {
        let handle_clone = self.handle.clone();
        return Callback::from(move |handle: UseStateHandle<T>| {
            handle_clone.set(Some(handle));
        });
    }
    pub fn get_state(&self) -> Option<UseStateHandle<T>> {
        return get_cell_data(&self.handle);
    }

    fn get_value(&self) -> T
    where
        T: Clone,
    {
        //初始化的值一定不会为空
        return get_cell_data(&self.value).unwrap();
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        return self
            .get_state()
            .map(|handle| handle.deref().clone())
            .unwrap_or_else(|| {
                return self.get_value();
            });
    }

    pub fn set(&self, data: T)
    where
        T: Clone,
    {
        if let Some(state) = self.get_state() {
            state.set(data.clone());
        }
        self.value.set(Some(data));
    }

    pub fn update_callback(&self) -> Callback<T>
    where
        T: 'static,
    {
        let handle_clone = self.handle.clone();
        return Callback::from(move |value: T| {
            let inner = handle_clone.take();
            if let Some(handle) = inner.as_ref() {
                handle.set(value);
            }
            handle_clone.set(inner);
        });
    }
    pub fn view(&self, renderer: impl UseStateRenderer<T> + 'static) -> Html
    where
        T: Clone + PartialEq + 'static,
    {
        binding_view(self, renderer)
    }
}

impl<T> From<UseStateHandle<T>> for Binding<T>
where
    T: Clone,
{
    fn from(value: UseStateHandle<T>) -> Self {
        Self {
            inherit_state: true,
            value: Arc::new(Cell::new(Some(value.deref().clone()))),
            handle: Arc::new(Cell::new(Some(value))),
        }
    }
}

impl<T> PartialEq<Self> for Binding<T>
where
    T: Clone + PartialEq<T>,
{
    fn eq(&self, other: &Self) -> bool {
        let state = self.get_state();
        let other_state = self.get_state();
        if state.is_none() && other_state.is_none() {
            return get_cell_data(&self.value) == get_cell_data(&other.value);
        } else {
            return state == other_state;
        }
    }
}

impl<T> Default for Binding<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct RequireUseStateProps<T: Clone + PartialEq> {
    pub init_value: T,
    pub oninit: Callback<UseStateHandle<T>>,
    pub renderer: ArcUseStateRenderer<T>,
    pub onchange: Callback<T>,
}

#[function_component]
pub fn RequireUseState<T>(props: &RequireUseStateProps<T>) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let item: UseStateHandle<T> = use_state(|| props.init_value.clone());
    let item_clone = item.clone();
    let oninit = props.oninit.clone();
    use_effect(move || {
        oninit.emit(item_clone);
        || ()
    });
    let onchange = props.onchange.clone();
    let item_clone1 = item.clone();
    let item_clone2 = item.clone();
    use_effect_with(item_clone1, move |_| {
        onchange.emit(item_clone2.deref().clone());
        || ()
    });
    return props.renderer.render(item);
}

pub trait UseStateRenderer<T> {
    fn render(&self, value: UseStateHandle<T>) -> Html;
}

impl<T, F> UseStateRenderer<T> for F
where
    F: Fn(UseStateHandle<T>) -> Html,
{
    fn render(&self, value: UseStateHandle<T>) -> Html {
        self(value)
    }
}

pub struct ArcUseStateRenderer<T> {
    pub inner: Arc<dyn UseStateRenderer<T>>,
}

impl<T, F: Fn(UseStateHandle<T>) -> Html + 'static> From<F> for ArcUseStateRenderer<T> {
    fn from(inner: F) -> Self {
        ArcUseStateRenderer {
            inner: Arc::new(inner),
        }
    }
}

impl<T> Clone for ArcUseStateRenderer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Deref for ArcUseStateRenderer<T> {
    type Target = dyn UseStateRenderer<T>;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[allow(clippy::vtable_address_comparisons)]
impl<T> PartialEq for ArcUseStateRenderer<T> {
    fn eq(&self, other: &ArcUseStateRenderer<T>) -> bool {
        let (ArcUseStateRenderer { inner }, ArcUseStateRenderer { inner: rhs }) = (self, other);
        Arc::ptr_eq(inner, rhs)
    }
}

impl<T> fmt::Debug for ArcUseStateRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcUseStateRenderer<_>")
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct ListenStateProps<T: Clone + PartialEq> {
    pub value: UseStateHandle<T>,
    pub onchange: Callback<T>,
    pub children: Children,
}

#[function_component]
pub fn ListenState<T>(props: &ListenStateProps<T>) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let onchange = props.onchange.clone();
    let value = props.value.clone();
    use_effect_with(props.value.clone(), move |_| {
        onchange.emit(value.deref().clone());
        || ()
    });
    return html! { props.children.clone() }
}

pub fn binding_view<T>(binding: &Binding<T>, renderer: impl UseStateRenderer<T> + 'static) -> Html
where
    T: Clone + PartialEq + 'static,
{
    let on_init = binding.init_callback();
    let renderer = ArcUseStateRenderer {
        inner: Arc::new(renderer),
    };
    let value = binding.value.clone();
    let onchange = Callback::from(move |new_value| {
        value.set(Some(new_value));
    });
    if binding.inherit_state {
        let state = binding.get_state().unwrap();
        html! {
            <ListenState<T> value={state.clone()} onchange={onchange}>
                {
                    renderer.render(state)
                }
            </ListenState<T>>
        }
    } else {
        html! {
            <RequireUseState<T> init_value={binding.get_value()} oninit={on_init} renderer={renderer} onchange={onchange}/>
        }
    }
}
