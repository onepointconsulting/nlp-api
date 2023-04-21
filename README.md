# NLP API

Small project used to show how you can create REST APIs using the rust-bert library.

## How to run the project

Run this command on the root folder.

```bash
cargo run --color=always --package nlp-api --bin nlp-api
```

## Which APIs are provided?

For now you can translate to multiple languages using a POST API to 

`http://localhost:7000/translate`

with the content:

```
{
    "orig_text": "The advantage of microfrontend is that it makes an application more flexible and easier to maintain.",
    "language": "de"
}
```

This produces this result in German:

```
{
    "orig_text": "The advantage of microfrontend is that it makes an application more flexible and easier to maintain.",
    "translation": " Der Vorteil von microfrontend ist, dass es eine Anwendung flexibler und einfacher zu pflegen macht."
}
```