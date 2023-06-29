
$Env:LIBTORCH="C:\Users\gilfe\miniconda3\envs\pytorch\Lib\site-packages\torch"
$env:PATH+=";C:\Users\gilfe\miniconda3\envs\pytorch\Lib\site-packages\torch\lib"
#cargo.exe clean
cargo.exe run --color=always --package nlp-api --bin nlp-api