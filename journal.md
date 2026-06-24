

## TODO

- [ ] Bayer matricing not working for now
- [ ] take delay offset in account
- [ ] index picture / create a hash with parameter
    - [ ] use the parameters as argument for input
- [ ] collumn
- [ ] latency compensation
- [ ] non square image broken
- [x] optimize signal constuction, don't redo it if the parameters haven't changed

## BUG

- [x] feedback broken
- [x] interleaved broken 
    count was update 1 by 1 instead of 4 by 4
- [x] preserve in interleaved broken
- [x] Diagonal black ray -> flush flagging problem, I reduced the modulo by one for bayer, which created the problem for the interleaved, weird, was probably the fact that the bayer modification and reset flag where not set at the same time

## Journal

Ok I remember What I was up to
How to split rows and collums to know when to reset the filters and delay to avoid spilling on the next line


I'll need a kind of matrix or something, like building a function from all the constraints then applying it to the signal

### Bayer construction

I added bayer construction. The problem with the bayer in regard to composite and interleaved is that it's not really made in to phase, I would tend to order and signalise at the same point. The main problem is that you lose the row / col index when ordering. What I tried right now is to alternate R & G at each pixel, then swap to G&B when going to the next row / collumn. It will probably cause some problem in the near futur for dematricing were colors won't be at the good place, but anyway I'll see.

One of the big problem with my structure is that adding more ordering mode (diagonal, etc...) might prove mighty difficult.

pass a reference ?
have a function for each color ?


~~I use u8 for x and y which is obviously wrong, to do~~

the program still compiles and works when using bayer
but don't trust it the actual function are just not plugged

I got a problem
I want to borrow different part of the stuct at the same time.
seems like rust devs have thought [about this](https://doc.rust-lang.org/nomicon/borrow-splitting.html)

I'll need to ask a rust pro

Let's unfuck bayer matricing for the nonante time.

currently I have a big diagonal at the middle that is flipped when modifying the bayer construction boolean positions

switching column to true fix the color problem on half the picture

ah, I think it must be the modulo that is offseted by one pixel per line, and because the image is square it makes a diagonal, it has no relation ship to x and y.

indeed modulo was not zero based.

lezgong it's fixed

Signal is not processed in bayer mode ???
image is not modified when changing parameters
maybe reset at each pixel ? check flag
nope, even it continous mode it doesn't work
was not indexing the correct picture lmao

## Not in the right angle 

I did a weird fix on the bayer at some point where I swapped X and Y in the peeking function, seems that now the problem is also preset 

+ interleaved color problem
+ composite has a diagonal in the middle for some reason
  
now that I swapped x and y, bayer suddenly has a weird blooming / saturation effect

seems like filter resonance is not neutral as 1, but more around 10

delay at 1 seems to almost fix the color, probably and index mistake somewhere

disabling processing just to be sure
k the problems definitely comes from the processing
problems come from the filter
aaaah, it's probably because I don't compensate the latency
I would need to throw all first answers from the sum of all latency or something like that

should I add some post processing to enhance the saturation / contrast ?

adding reverb, no result with allpasses, prabably clipping somewhere or something. comb *has* an effect.
reverb sample rate around 4000 give 400 pixel delay lines
too much, 300 sample rate is actually way better, 100 even, lower than that I cannot instantiate the delay line, the max time is not handled super well

i'll need to add an init somewhere for the continuous thing

the rt60 is surprisingly sensible, 1 already give you color noise. above 0.2 is not really usable

## bayer color broken again

I'm going crazy
seems a lot like the green and red are swapped
swapping it at dematricing make like look almost correct, but the picture is heavily desaturated
I tried on touch, and indeed it seems like the green and red are swapped, but they're swapped only on one of the two green pixels I think, like if one row of two had a 1 pixel offset

okay, back with the blue rose

omg it's back
weiiiird, i needed to swap both the rows of the matrix and the bayer boolean used to signalize. didn't really understood what I just did

....
I think I just build the picture in composite by accident
fuuuuuuuuuuuuuu

there mmust be an incoherence between the bayer color selected at construction, and the bayer color selected at dematricing

ok count % modulo = 0 was trigging at the first occurence, pretty dumb mistake, should be better
rose blue again
also, the matrix is flipped in terms of rows
like array[x][y] is actually array[y][x]
because it's instantiated as [[0, 1], [2, 0]]
thus the first index selects the y, and the second the x

## WASM

I refactored everything to work within a js file
next step is building said js file with a canvas, a bunch of slider and a process button
I should be able to get most of the functional and UI things from the wepgpu repo made with guk

18 Jun Intefaced the js with the rust, you can load the image to the canvas and trigger the processing, but currently I'm struggling to output the image to the canvas
my output image seems to have the correct width, size and lenght, but I'm copying the value from the input source, I've got no proof that the data is right
try managing everything in the rust to be sure that there is no problem with the js glue between the processing and the canvas modification

try to make a dummy image a fill with white, put the put image definitely doesn't work, I need more info about the image format to be sure that I'm giving it something that makes sense.

>ImageData.data Read only
>    A Uint8ClampedArray or Float16Array representing a one-dimensional array containing the data in the RGBA order. The order goes by rows from the top-left pixel to the bottom-right.

seems correct [from here](https://developer.mozilla.org/en-US/docs/Web/API/ImageData)

try to do the put image data in js ? actually it's exactly the same thing that happen in rust

ok, white image to canvas in rust works , I fill the picture with white pixel at a lower level in the processor to check the link between main and the processor, if the problem doesn't come from the js-wasm glue. the problem for the white picture came from using the wrong size, it failed silently it seems, I added and `asserteq()` to check the size but it seems to work ok with the processor, next step, check the pixel content of the output picture vector

Ok, indeed weird stuff is happening, only the first quarter get filled with white, which would indicate some kind of iteration problem somewhere

let's try to convert the incoming data in a picture, then converting it back, to be sure that the class constructor works properly

Doing the smallest possible code that reproduce the bug, it's still present, it's probably coming from the set_pixel function

it's always exactly one quarter which is very weird
probably linked with the 4 colors ?
one pixel setting and getting seems to work properly though

BUT isn't it that the index if broken in the two sides and thus showing two time the wrong pixel ?

lmao I just needed to change the get index function, it didn't took into account the fact that everything is multiplied by four

filter is broken but the rest actually works !!

let's add sliders

sliders works, currently, once you processed you cannot change the picture anymore, the processing is incremental, which is not the point (although it's an interesting feature)

Made a hardwired wavefolder
$y = sin(x/10)*127+127$
made to use full range
I'll need to add a kind of param on the frequency on the wavefolder (the ten) and the amount of it
I could do an interpolation between this output and the actual sample (a dry,wet, actually)

right now to add a param you need to : 
1. add it in the parameters description
2. add it in the parameters constructor (argument + function)
4. add it to the main function parameter construction
5. add it to the html file
6. retrieve value in js

do I even need to have the parameter struct still ?
it mainly useful for UI stuff but it's not really relevant anymore


## handling latency

    the thing is to still fill the image completely
    the while / for loop / counter should refer to the destination picture, i'll need to pad the end of the row of pixel to get all values out

    dirsty : at reset, first push latency_amount of pixel to dest, then pre process the amount

    will create but if latency > image size

    slowly getting there, currently I have an offset of delay_time size at the first line, then a 1 pixel offset building up (increment error) 