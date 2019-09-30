use jni_android_sys::java::lang::Throwable;
use jni_glue::Local;

pub type Result<T> = std::result::Result<T, String>;

pub struct ResultOption<T, E>(std::result::Result<Option<T>, E>);

impl<T, U> From<std::result::Result<Option<T>, U>> for ResultOption<T, U> {
    fn from(val: std::result::Result<Option<T>, U>) -> ResultOption<T, U> {
        ResultOption(val)
    }
}

impl<'env, T> Into<Result<T>> for ResultOption<T, Local<'env, Throwable>> {
    fn into(self) -> Result<T> {
        match self.0 {
            Ok(Some(y)) => Ok(y),
            Ok(None) => Err(format!("Java function returned NULL - {}:{}", file!(), line!())),
            Err(e) => Err(format!("{:?}", e.toString().unwrap().unwrap())),
        }
    }
}

#[macro_export]
macro_rules! resopt {
    ( $x:expr ) => {
        ResultOption::from($x).into()
    };
}

#[macro_export]
macro_rules! stringify_throwable {
    ( $x:expr ) => {
        $x.map_err(|e| format!("{:?}", e.toString().unwrap().unwrap()))
    };
}

