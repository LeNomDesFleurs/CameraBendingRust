// import { state, getRenderDonePromise, SetBitMap, IMAGE_URL } from './src/utils.js'
// import rust_wasm_init from "./pkg/bending.js";

// Source - https://stackoverflow.com/a/74497796
// Posted by Paco Wong
// Retrieved 2026-06-18, License - CC BY-SA 4.0

// async function run_wasm_function() {
//     const path_to_wasm = "./pkg/bending.js'"; //Remember to update the parameter in CopyPlugin
//     await rust_wasm_init(path_to_wasm); //This initializes the wasm object mentioned above
//     const foo = new Foo();
//     console.log(foo.get_contents());
// }

// run_wasm_function()

import init, { process_picture, putImageData } from './pkg/bending.js'

let CURRENT_MODEL
let renderContext

var download = function () {}

let source_image = new Image()

async function main() {
    await init() // must come first

    var imageLoader = document.getElementById('imageLoader')
    imageLoader.addEventListener('input', handleImage, false)
    var canvas = document.getElementById('imageCanvas')
    var ctx = canvas.getContext('2d')

    var processButton = document.getElementById('process')
    processButton.addEventListener('click', async () => {
        let raw_data = process_picture(
            canvas,
            ctx,
            document.querySelector('input[name="alpha_mode"]:checked').value, //alpha mode
            document.querySelector('input[name="color_mode"]:checked').value, //color mode
            document.querySelector('input[name="order_mode"]:checked').value, //order mode
            document.getElementById('delay').value, //delay time
            document.getElementById('feedback').value, //delay feedback
            document.getElementById('filter_cutoff').value, // filter cutoff
            document.getElementById('filter_resonance').value, // filter resonance
            document.getElementById('reverb_time').value, //reverb time
            0.0, //reverb dry wet
            document.getElementById('wavefolder_amount').value,
            document.getElementById('wavefolder_freq').value,
            document.getElementById('bitwise').value,

            document.getElementById('continous').checked //continous
        )

        console.log(document.getElementById('feedback').value)
    })

    var downloadButton = document.getElementById('download')
    downloadButton.addEventListener('click', async () => {
        var link = document.createElement('a')
        link.download = 'processed.png'
        link.href = canvas.toDataURL()
        link.click()
    })

    var ResetButton = document.getElementById('reset')

    ResetButton.addEventListener('click', async () => {
        ctx.drawImage(source_image, 0, 0)
    })

    function handleImage(e) {
        var reader = new FileReader()
        reader.onload = function (event) {
            source_image.onload = function () {
                canvas.width = source_image.width
                canvas.height = source_image.height
                ctx.drawImage(source_image, 0, 0)
            }
            source_image.src = event.target.result
        }
        reader.readAsDataURL(e.target.files[0])
    }
}

main()

// document.addEventListener('DOMContentLoaded', async () => {

//     const canvas = document.getElementById(CANVAS_ID);
// const img = new Image(); // Create new img element
// img.src = "./assets/rose.jpg"; // Set source path
//     const modelsContainer = document.getElementById('models-container');
//         const button = document.createElement('button')
//         button.textContent = "Params";
//         button.style.backgroundColor = 'rgb(255, 255, 255)';
//         button.style.opacity = 0.7;
//         button.style.margin = '0.1em';
//         button.style.width = '5rem';
//         button.style.border = '0em';
//         button.style.borderRadius = '1em'
//         button.addEventListener('click', async () => {
//             // if (CURRENT_MODEL.name == model.name) { plus tard -> disable if already clicked
//             //     button.style.backgroundColor = 'green';
//             //     button.disabled = true;
//             // }
//             CURRENT_MODEL.destroy();

//             CURRENT_MODEL = new model(renderContext.getDevice(), renderContext);
//             await CURRENT_MODEL.init();
//             CURRENT_MODEL.render();
//         })
//         modelsContainer.appendChild(button);

//         async function handeDownload(pixelBuffer, widthBytes, heightBytes) {
//             let canvas = document.getElementById('gfx')
//             let canvasUrl = canvas.toDataURL('image/jpeg', 1)
//             const createEl = document.createElement('a')
//             createEl.href = canvasUrl
//             createEl.download = 'Processed Image'
//             createEl.click()
//             createEl.remove()
//         }

//         // <label>PNG file: <input type="file" id="image_input" accept="image/png" id="load-image"></label>
//         const container = document.getElementById('controller')
//         const fileLabel = document.createElement('label')
//         // fileLabel.textContent = "PNG file: "
//         const input = document.createElement('input')
//         input.type = 'file'
//         input.accept = 'image/png, image/jpg, image/jpeg'
//         input.addEventListener('change', async (event) => {
//         const file = event.target.files[0]
//         IMAGE_URL = file;
//         // create a temp. image object
//         this.render()
//     })
//     fileLabel.appendChild(input)
//     container.appendChild(fileLabel)

//     const DownloadButton = document.createElement('button')
//         ; (DownloadButton.textContent = 'Download'),
//             (DownloadButton.style.width = '100px')
//     DownloadButton.addEventListener('click', async (event) => {
//         await this.render()
//         handeDownload(this.pixelBuffer, this.widthBytes, this.heightBytes)
//     })
//     modelsContainer.appendChild(DownloadButton)
//     // <button id="download">Download Canvas</button>

//     function convertBGRAtoRGBA(input, width, height) {
//         const output = new Uint8ClampedArray(width * height * 4)
//         for (let i = 0; i < width * height; i++) {
//             const idx = i * 4
//             output[idx + 0] = input[idx + 2] // R <- B
//             output[idx + 1] = input[idx + 1] // G <- G
//             output[idx + 2] = input[idx + 0] // B <- R
//             output[idx + 3] = input[idx + 3] // A unchanged
//         }
//         return output
//     }

// });
