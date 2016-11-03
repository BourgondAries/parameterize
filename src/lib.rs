#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
//! Tramp/Parameterize is a library for avoiding
//! tramp data.
//! Tramp data is pass-through data not used by intermediary functions:
//!
//! ```ignore
//! fn f(a: i32) { g(a); };
//! fn g(a: i32) { h(a); };
//! fn h(a: i32) { i(a); };
//! fn i(a: i32) { println!["Only I use {}", a]; };
//!
//! fn main() {
//!   f(10);
//! }
//! ```
//!
//! A cleaner way of doing this may be:
//!
//! ```ignore
//! thread_local! { static A: RefCell<i32> = RefCell::new(0); }
//! fn f() { g(); };
//! fn g() { h(); };
//! fn h() { A.with(|a| i(*a.borrow())); };
//! fn i(a: i32) { println!["Only I use {}", a]; };
//!
//! fn main() {
//!   tramp! { A: 10 => f() };
//! }
//!
//! ```
//!
//! This is useful for deep call chains when objects can't store the value for you. The intermediate functions are much cleaner.

#[macro_use]
extern crate scopeguard;

#[macro_export]
macro_rules! tramp {
	($ih:ident : $eh:expr, $($it:ident : $et:expr),* => $b:block) => { {
			let old = $ih.with(|x| {
				let old = x.borrow().clone();
				*x.borrow_mut() = $eh;
				old
			});
			defer![$ih.with(|x| { *x.borrow_mut() = old.clone(); })];
			tramp![$($it : $et),* => $b];
		}
	};

	($ih:ident : $eh:expr => $b:block) => {
		let old = $ih.with(|x| {
			let old = x.borrow().clone();
			*x.borrow_mut() = $eh;
			old
		});
		{
			defer![$ih.with(|x| { *x.borrow_mut() = old.clone(); })];
			$b
		}
	};
}

#[cfg(test)]
mod tests {

	use std::cell::RefCell;

	thread_local! {
		pub static FOO: RefCell<u32> = RefCell::new(0);
		pub static BAR: RefCell<String> = RefCell::new(String::new());
	}

	#[test]
	fn single_variable() {

		tramp! { FOO: 100 => {
			FOO.with(|x| {
			BAR.with(|y| {
				assert_eq![*x.borrow(), 100];
				assert_eq![*y.borrow(), ""];
			})});
		}}

		FOO.with(|x| {
		BAR.with(|y| {
			assert_eq![*x.borrow(), 0];
			assert_eq![*y.borrow(), ""];
		})});

	}

	#[test]
	fn two_variables() {

		tramp! { FOO: 1, BAR: "A".to_string() => {
			FOO.with(|x| {
			BAR.with(|y| {
				assert_eq![*x.borrow(), 1];
				assert_eq![*y.borrow(), "A"];
			})});
		}}

		FOO.with(|x| {
		BAR.with(|y| {
			assert_eq![*x.borrow(), 0];
			assert_eq![*y.borrow(), ""];
		})});
	}

}
