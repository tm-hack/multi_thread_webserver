# multi_thread_webserver

## Overview
The Rust Programming Language 日本語版 20章の演習内容です。

https://doc.rust-jp.rs/book-ja/ch20-00-final-project-a-web-server.html


## memo
lib.rsの以下の記載についてドキュメントのままだと、コンパイラに怒られてしまう。
```rust
type Job = Box<FnOnce() + Send + 'static>;
```

```bash
trait objects must include the `dyn` keyword
```

このエラーについては以下のように修正するとうまくいく。
理由は不明。

* impl<F: FnOnce()> ~ をimpl <F: FnOnce() + ?Sized> ~ に修正する
* type Job = Box<FnOnce() ~ をtype Job = Box<dyn FnOnce() ~ に修正する

```rust
impl<F: FnOnce() + ?Sized> FnBox for F {
    fn call_box(self: Box<F>) {
        (self)()
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
```

ちなみにimpl<F: FnOnce()> ~ のままだと以下のエラーを出力する。

```bash
the method `call_box` exists for struct `Box<dyn FnOnce() + Send>`, but its trait bounds were not satisfied
method cannot be called on `Box<dyn FnOnce() + Send>` due to unsatisfied trait bounds

構造体 `Box<dyn FnOnce() + Send>` に対してメソッド `call_box` は存在するが、その特性値が満たされていない。
Box<dyn FnOnce() + Send>` に対して、特性境界が満たされていないため、メソッドを呼び出すことができません。
```

type Job = Box<FnOnce() ~ のままだと以下のエラーを出力する。

```bash
trait objects must include the `dyn` keyword
trait オブジェクトは `dyn` キーワードを含んでいなければなりません。
```

参考にしたのはstack overflowで見つけた以下の質問である。

[Why is trait implemented on a type with some trait bound not accepting functions implemented on them? [duplicate]](https://stackoverflow.com/questions/57311728/why-is-trait-implemented-on-a-type-with-some-trait-bound-not-accepting-functions)

I think the problem is the implicit Sized bound on F in the impl for your FnBox trait, which makes a Box<dyn T> not covered under that impl.

You say

> The trait FnBox has been implemented on all the types with FnOnce() trait.

But actually the trait FnBox has been implemented for only all Sized types with FnOnce() trait. The docs for Sized have more info about this.

A working example is:

```rust
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce() + ?Sized> FnBox for F {
    fn call_box(self: Box<F>) {
        (self)()
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;

let job: Job = Box::new(|| println!("gwahhh"));
job.call_box();
```

Note that I had to remove the (*self)() in favor of (self)() because you cant move an unsized type out of a Box.
