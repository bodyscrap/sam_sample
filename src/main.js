const { invoke } = window.__TAURI__.core;
const { open, save } = window.__TAURI__.dialog;

let _imageCanvas = document.querySelector("#imageCanvas");
let _maskCanvas = document.querySelector("#maskCanvas");
let _selectCanvas = document.querySelector("#selectCanvas");
let _bufImage = _imageCanvas.getContext("2d");
let _bufMask = _maskCanvas.getContext("2d");
let selectedPos = null;
let isImageLoaded = false;

// DOMの読み込み完了後に行う処理
window.addEventListener("DOMContentLoaded", () => {
  // イベントの登録
  document.querySelector("#btnOpen").addEventListener("click", openImage);
  document.querySelector("#btnProc").addEventListener("click", showMask);
  _selectCanvas.addEventListener("click", selectPos);
});

// ハンドラ

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
    // 画像バッファを作成(RGBA 初期値は (0,0,0,0))
    let img = _bufImage.createImageData(_imageCanvas.width, _imageCanvas.height);
    // Rust側から受け取ったRGBPデータをコピー
    for (let i = 0; i < img.data.length; i++ ) {
      img.data[i] = data[2][i]
    }
    // 準備完了したバッファをCanvasのバッファに設定
    _bufImage.putImageData(img,0,0);
    // 選択解除
    clearMarker(selectedPos);
    selectedPos = null;
    isImageLoaded = true;
  });
}

async function showMask() 
{
  if(isImageLoaded == true && selectedPos != null)
  {
    // canvasのサイズの取得
    let w = _imageCanvas.width;
    let h = _imageCanvas.height;
    // canvasの画像バッファを取得
    let buf = _bufImage.getImageData(0, 0, _imageCanvas.width, _imageCanvas.height);
    let result = await invoke("process_sam", 
      { 
        width: w, 
        height: h, 
        data: Array.from(buf.data),
        px: selectedPos.x,
        py: selectedPos.y,
      });
    // 画像バッファを作成(RGBA 初期値は (0,0,0,0))
    let buf_mask = _bufMask.createImageData(_maskCanvas.width, _maskCanvas.height);
    // Rust側から受け取ったRGBPデータをコピー
    for (let i = 0; i < buf_mask.data.length; i++ ) {
      buf_mask.data[i] = result[2][i]
    }
    // 準備完了したバッファをCanvasのバッファに設定
    _bufMask.putImageData(buf_mask, 0, 0);
  }
}

// その他のメソッド

function selectPos(e)
{
  if(isImageLoaded == true) {
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