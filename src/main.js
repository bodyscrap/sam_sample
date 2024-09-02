const { invoke } = window.__TAURI__.tauri;
const { open, save } = window.__TAURI__.dialog;

let _imageCanvas = document.querySelector("#imageCanvas");
let _maskCanvas = document.querySelector("#maskCanvas");
let _selectCanvas = document.querySelector("#selectCanvas");
let _bufImage = _imageCanvas.getContext("2d");
let selectedPos = null;
let isImageLodaed = false;

// DOMの読み込み完了後に行う処理
window.addEventListener("DOMContentLoaded", () => {
  // イベントの登録
  document.querySelector("#btnOpen").addEventListener("click", openImage);
  _selectCanvas.addEventListener("click", selectPos);
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
    // 選択解除
    clearMarker(selectedPos);
    selectPos = null;
    isImageLodaed = true;
  });
}

function selectPos(e)
{
  if(isImageLodaed == true) {
    let rect = e.target.getBoundingClientRect();
    let x = e.clientX - rect.left;
    let y = e.clientY - rect.top;
    let pos = { x: x, y: y };
    console.log(pos);
    clearMarker(selectedPos);
    drawMarker(pos, selectedPos);
    selectedPos = pos
  }
}

function drawMarker(pos)
{
  let ctx = _selectCanvas.getContext("2d");
  ctx.beginPath();
  ctx.arc(pos.x, pos.y, 5, 0, Math.PI * 2, false);
  ctx.fillStyle = "red";
  ctx.fill();
  ctx.closePath();
  ctx.stroke();
}

function clearMarker(pos)
{
  // 前回の円をカバーする範囲を透明な矩形でクリア
  if (pos != null) {
    let ctx = _selectCanvas.getContext("2d");
    ctx.clearRect(pos.x - 7, pos.y - 7, 14, 14);
  }
}