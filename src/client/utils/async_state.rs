#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState<T, E> {
    Idle,
    Loading,
    Ready(T),
    Error(E),
}

impl<T, E> LoadState<T, E> {
    pub fn idle() -> Self {
        Self::Idle
    }

    pub fn loading() -> Self {
        Self::Loading
    }

    pub fn ready(value: T) -> Self {
        Self::Ready(value)
    }

    pub fn error(error: E) -> Self {
        Self::Error(error)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn as_ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_error(&self) -> Option<&E> {
        match self {
            Self::Error(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationState<E> {
    Idle,
    Pending,
    Success,
    Error(E),
}

impl<E> MutationState<E> {
    pub fn idle() -> Self {
        Self::Idle
    }

    pub fn pending() -> Self {
        Self::Pending
    }

    pub fn success() -> Self {
        Self::Success
    }

    pub fn error(error: E) -> Self {
        Self::Error(error)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn as_error(&self) -> Option<&E> {
        match self {
            Self::Error(error) => Some(error),
            _ => None,
        }
    }
}
