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

with the body content:

```json
{
    "orig_text": "The advantage of microfrontend is that it makes an application more flexible and easier to maintain.",
    "language": "de"
}
```

This produces this result in German:
```json
{
    "orig_text": "The advantage of microfrontend is that it makes an application more flexible and easier to maintain.",
    "translation": " Der Vorteil von microfrontend ist, dass es eine Anwendung flexibler und einfacher zu pflegen macht."
}
```

Another API is Zero shot classification using also the POST method:

`http://localhost:7000/zero_shot`

With this example content:

```json
{
  "orig_text": "Take a chance and commit: dating apps and streaming television now operate along quite a similar structure: endless options for you to consider at a glance, with nothing further to point you to the right one. The difference with streaming is that you don’t need to wait for approval from the other side to get started: if a show or film catches your eye, stop and see if you like it. Too many of us waste hours shuffling through menus in search of some ideal viewing option that we can’t even define for ourselves, instead of just choosing on a whim. Worst-case scenario? You tune out after 15 minutes and start again.",
  "split": false,
  "labels": ["entertainment", "public health", "media", "internet"]
}
```

You get this response:

```json
{
  "sentences": [
    "Take a chance and commit: dating apps and streaming television now operate along quite a similar structure: endless options for you to consider at a glance, with nothing further to point you to the right one. The difference with streaming is that you don’t need to wait for approval from the other side to get started: if a show or film catches your eye, stop and see if you like it. Too many of us waste hours shuffling through menus in search of some ideal viewing option that we can’t even define for ourselves, instead of just choosing on a whim. Worst-case scenario? You tune out after 15 minutes and start again."
  ],
  "responses": [
    [
      {
        "text": "entertainment",
        "score": 0.5741154551506042,
        "id": 0,
        "sentence": 0
      },
      {
        "text": "public health",
        "score": 0.07423999160528183,
        "id": 1,
        "sentence": 0
      },
      {
        "text": "media",
        "score": 0.9658299684524536,
        "id": 2,
        "sentence": 0
      },
      {
        "text": "internet",
        "score": 0.6292471289634705,
        "id": 3,
        "sentence": 0
      }
    ]
  ],
  "status": "OK"
}
```

Summarization and keyword generation are also supported.
