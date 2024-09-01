const { invoke } = window.__TAURI__.tauri;
const { open, save } = window.__TAURI__.dialog;

let _imageCanvas = document.querySelector("#imageCanvas");
let _maskCanvas = document.querySelector("#maskCanvas");
let _selectCanvas = document.querySelector("#selectCanvas");
let _bufImage = _imageCanvas.getContext("2d");

// DOMの読み込み完了後に行う処理
window.addEventListener("DOMContentLoaded", () => {
  // イベントの登録
  document.querySelector("#btnOpen").addEventListener("click", openImage);
});

function openImage() 
{
  open({
    filters: [
      { name: 'PNG', extensions: ['png'] },
      { name: 'JPG', extensions: ['jpg'] },
      { name: 'Bitmap', extensions: ['bmp'] },
    ],
  }).then(async file => {
    let data = await invoke("open_img", { path: file });
    _imageCanvas.width = data[0];
    _imageCanvas.height = data[1];
    _maskCanvas.width = data[0];
    _maskCanvas.height = data[1];
    _selectCanvas.width = data[0];
    _selectCanvas.height = data[1];
    let img = _bufImage.createImageData(_imageCanvas.width, _imageCanvas.height);
    for (let i = 0; i < img.data.length; i++ ) {
      img.data[i] = data[2][i]
    }
    _bufImage.putImageData(img,0,0);
  });
}