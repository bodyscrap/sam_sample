//! SegmentAnythingのラッパーモジュール
use tauri::{App, Manager};
use anyhow::Result; // Resultのラッパーユーティリティ
use candle_core::{DType, Device, Tensor};
use candle_nn::var_builder::VarBuilder;
use candle_transformers::models::segment_anything::sam::{self, Sam};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::sync::Mutex;
use std::vec;

/// SegmentAnythingを利用したアプリケーション
pub struct SamApp {
    /// モデル
    model: Mutex<Sam>,
    /// 推論デバイス
    device: Mutex<Device>,
    /// マスクの色
    mask_color: Mutex<Rgba<u8>>,
}

impl SamApp {
    /// tyny SAMでの初期化
    /// # Arguments
    /// - `model_path` : モデルのパス(本サンプルでは未使用)
    /// # Returns
    /// SamAppのインスタンス
    pub fn new_tyny(app:&mut App, model_path: &str) -> Result<Self> {
        // デバイスの取得(Cudaが使えるならGPU0を使用。そうでないならCPUを使用)
        let device = match Device::new_cuda(0) {
            Ok(device) => device,
            Err(_) => Device::Cpu,
        };
        // モデルのパラメータのロード
        let resource_dir = app.path().resource_dir().expect("failed to resolve resource");
        let model_data = resource_dir.join(model_path);
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[model_data], DType::F32, &device)? };

        // 指定のVarBuilderで初期化したモデルを作成
        let model = Sam::new_tiny(vb)?;
        // インスタンスを初期化
        Ok(Self {
            model: Mutex::new(model),
            device: Mutex::new(device),
            mask_color: Mutex::new(Rgba([255u8, 0u8, 0u8, 128u8])),
        })
    }

    /// 入力画像からマスクを生成
    /// # Arguments
    /// - `width` : 画像の幅
    /// - `height` : 画像の高さ
    /// - `img_data` : RGBA画像のバッファ
    /// - `px` : positive point のx座標
    /// - `py` : positive point のy座標
    /// # Returns
    /// 前景マスク画像バッファ
    pub fn process_sam(
        &self,
        width: u32,
        height: u32,
        img_data: Vec<u8>,
        px: f32,
        py: f32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        // 設定されているマスクの色を取得
        let pos_point = vec![(px as f64 / width as f64, py as f64 / height as f64, true)];
        // u8配列からImageBufferを作成
        let img: Option<ImageBuffer<Rgba<u8>, Vec<u8>>> =
            ImageBuffer::from_vec(width, height, img_data);
        match img {
            Some(img) => {
                let img = DynamicImage::ImageRgba8(img);
                // モデルに合わせてリサイズしたTensorと元の画像サイズを取得
                let (input, org_h, org_w) = self.make_input_tensor(img)?;
                let model = self.model.lock().unwrap();
                let (mask, _iou_prediction) = model.forward(&input, &pos_point, false)?;
                let mask_buf = self.create_mask(mask, org_w, org_h)?;
                Ok(mask_buf)
            }
            None => Err(anyhow::anyhow!("Failed to create ImageBuffer")),
        }
    }

    #[cfg_attr(doc, katexit::katexit)] // ドキュメント生成時にkatexitを適用
    /// 入力画像からネットワークへの入力Tensorを作成。  
    /// 入力画像の幅、高さの内長い方が[sam::IMAGE_SIZE]になるようにリサイズ  
    /// 試しに書いて見ただけの数式 $\sum_{i=0}^{k} x^i$ 。
    /// # Arguments
    /// - `img` : 入力画像
    /// # Returns
    /// - 入力Tensor
    /// - 入力画像の高さ(pixel)
    /// - 入力画像の幅(pixel)
    fn make_input_tensor(&self, img: DynamicImage) -> Result<(Tensor, usize, usize)> {
        // 入力画像サイズの保存
        let (initial_h, initial_w) = (img.height() as usize, img.width() as usize);
        // 画像の長い側の辺がsam::IMAGE_SIZEになるようにアス比を保ってリサイズ
        let (height, width) = (img.height(), img.width());
        let resize_longest = sam::IMAGE_SIZE as u32;
        let (height, width) = if height < width {
            let h = (resize_longest * height) / width;
            (h, resize_longest)
        } else {
            let w = (resize_longest * width) / height;
            (resize_longest, w)
        };
        // リサイズ
        let img = img.resize_exact(width, height, image::imageops::FilterType::CatmullRom);
        // リサイズ後のサイズを取得
        let (height, width) = (img.height() as usize, img.width() as usize);
        // リサイズ後の画像をRGBに変換し、Tensorを作成
        let img = img.to_rgb8();
        let data = img.into_raw();
        let data = Tensor::from_vec(data, (height, width, 3), &self.device.lock().unwrap())?
            .permute((2, 0, 1))?; // Pytorchの形式である(C, H, W)に変換
        Ok((data, initial_h, initial_w))
    }

    /// マスク画像を作成
    /// # Arguments
    /// - `input` : マスクの入力Tensor
    /// - `out_w` : 出力画像の幅
    /// - `out_h` : 出力画像の高さ
    /// # Returns
    /// マスク画像バッファ
    fn create_mask(
        &self,
        input: Tensor,
        out_w: usize,
        out_h: usize,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let color = self.mask_color.lock().unwrap();
        let _temp_dim = input.dims();
        let h = _temp_dim[1];
        let w = _temp_dim[2];
        // RgbaImageを作成 (幅w, 高さhのRGBA画像)
        let mut img = RgbaImage::new(w as u32, h as u32);
        // Tensorをフラット化して参照
        let values = input
            .flatten_all()?
            .to_vec1::<f32>()
            .expect("Failed to convert tensor to Vec<f32>");
        let th = 0.5f32;
        // 閾値以上の画素に色付け
        for y in 0..h {
            for x in 0..w {
                let index = y * w + x;
                let value = values[index];
                let pixel = if value >= th {
                    color.clone() // マスクの色
                } else {
                    Rgba([0, 0, 0, 0]) // 透明
                };
                // ピクセルを画像に設定
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }
        // リサイズして(outH, outW, 4)に変換
        let img = image::imageops::resize(
            &img,
            out_w as u32,
            out_h as u32,
            image::imageops::FilterType::Nearest,
        );
        Ok(img)
    }
}
