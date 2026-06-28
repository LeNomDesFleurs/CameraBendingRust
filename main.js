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
    ctx.imageSmoothingEnabled = false;

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
            document.getElementById('reverb_dry_wet').value, //reverb dry wet
            document.getElementById('reverb_time').value, //reverb time
            document.getElementById('reverb_size').value, //reverb dry wet
            document.getElementById('wavefolder_amount').value,
            document.getElementById('wavefolder_freq').value,
            document.getElementById('bitwise').value,

            document.getElementById('continous').checked //continous
        )

        console.log(document.getElementById('reverb_size').value)
        console.log(document.getElementById('reverb_time').value)
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
