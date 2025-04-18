//! afl plugin for bolero
//!
//! This crate should not be used directly. Instead, use `bolero`.

#[doc(hidden)]
#[cfg(any(test, all(feature = "lib", fuzzing_afl)))]
pub mod fuzzer {
    use bolero_engine::{driver, input, panic, Engine, Never, ScopedEngine, TargetLocation, Test};
    use std::io::Read;

    extern "C" {
        // from the afl-llvm-rt
        fn __afl_persistent_loop(counter: usize) -> isize;
        fn __afl_manual_init();
    }

    #[used]
    static PERSIST_MARKER: &str = "##SIG_AFL_PERSISTENT##\0";

    #[used]
    static DEFERED_MARKER: &str = "##SIG_AFL_DEFER_FORKSRV##\0";

    #[derive(Debug, Default)]
    pub struct AflEngine {}

    impl AflEngine {
        pub fn new(_location: TargetLocation) -> Self {
            Self::default()
        }
    }

    impl<T: Test> Engine<T> for AflEngine
    where
        T::Value: core::fmt::Debug,
    {
        type Output = Never;

        fn run(self, mut test: T, options: driver::Options) -> Self::Output {
            panic::set_hook();

            let mut input = AflInput::new(options);

            unsafe {
                __afl_manual_init();
            }

            while unsafe { __afl_persistent_loop(1000) } != 0 {
                if test.test(&mut input.test_input()).is_err() {
                    std::process::abort();
                }
            }

            std::process::exit(0);
        }
    }

    impl ScopedEngine for AflEngine {
        type Output = Never;

        fn run<F, R>(self, mut test: F, options: driver::Options) -> Self::Output
        where
            F: FnMut() -> R + core::panic::RefUnwindSafe,
            R: bolero_engine::IntoResult,
        {
            panic::set_hook();

            // extend the lifetime of the bytes so it can be stored in local storage
            let driver = bolero_engine::driver::bytes::Driver::new(vec![], &options);
            let driver = bolero_engine::driver::object::Object(driver);
            let mut driver = Box::new(driver);

            let mut input = AflInput::new(options);

            unsafe {
                __afl_manual_init();
            }

            while unsafe { __afl_persistent_loop(1000) } != 0 {
                input.reset();
                let bytes = core::mem::take(&mut input.input);
                let tmp = driver.reset(bytes, &input.options);
                let (drv, result) = bolero_engine::any::run(driver, &mut test);
                driver = drv;
                input.input = driver.reset(tmp, &input.options);

                if result.is_err() {
                    std::process::abort();
                }
            }

            std::process::exit(0);
        }
    }

    #[derive(Debug)]
    pub struct AflInput {
        options: driver::Options,
        input: Vec<u8>,
    }

    impl AflInput {
        fn new(options: driver::Options) -> Self {
            Self {
                options,
                input: vec![],
            }
        }

        fn reset(&mut self) {
            self.input.clear();
            std::io::stdin()
                .read_to_end(&mut self.input)
                .expect("could not read next input");
        }

        fn test_input(&mut self) -> input::Bytes {
            self.reset();
            input::Bytes::new(&self.input, &self.options)
        }
    }
}

#[doc(hidden)]
#[cfg(all(feature = "lib", fuzzing_afl))]
pub use fuzzer::*;

#[doc(hidden)]
#[cfg(feature = "bin")]
pub mod bin {
    use std::{
        ffi::CString,
        os::raw::{c_char, c_int},
    };

    extern "C" {
        // entrypoint for afl
        pub fn afl_fuzz_main(a: c_int, b: *const *const c_char) -> c_int;
    }

    /// Should only be used by `cargo-bolero`
    ///
    /// # Safety
    ///
    /// Use `cargo-bolero`
    pub unsafe fn exec<Args: Iterator<Item = String>>(args: Args) {
        // create a vector of zero terminated strings
        let args = args
            .map(|arg| CString::new(arg).unwrap())
            .collect::<Vec<_>>();

        // convert the strings to raw pointers
        let c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .chain(Some(core::ptr::null())) // add a null pointer to the end
            .collect::<Vec<_>>();

        let status = afl_fuzz_main(args.len() as c_int, c_args.as_ptr());
        if status != 0 {
            std::process::exit(status);
        }
    }
}

#[doc(hidden)]
#[cfg(feature = "bin")]
pub use bin::*;
