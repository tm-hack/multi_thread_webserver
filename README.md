# single_thread_webserver

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

* <F: FnOnce()>を<F: FnOnce() + ?Sized>に修正する
* Box<FnOnce() をBox<dyn FnOnce() に修正する

```rust
impl<F: FnOnce() + ?Sized> FnBox for F {
    fn call_box(self: Box<F>) {
        (self)()
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
```

参考にしたのはstack overflowで見つけた以下の質問である。

[Why is trait implemented on a type with some trait bound not accepting functions implemented on them? [duplicate]](https://stackoverflow.com/questions/57311728/why-is-trait-implemented-on-a-type-with-some-trait-bound-not-accepting-functions)


