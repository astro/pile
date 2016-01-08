'use strict'

const FPS = 60
const LEDS = 20 + 206
const HOST = 'ledbeere.hq.c3d2.de'
const PORT = 2342

let dgram = require('dgram')
let sock = dgram.createSocket('udp4')

let buf = new Buffer(4 + 3 * LEDS)
buf[0] = 0  // channel: 0
buf[1] = 0  // command: CMD_SET_PIXEL_COLORS
buf.writeUIntBE(3 * LEDS, 2, 2)

function send(pixels) {
    for(var i = 0; i < LEDS; i++) {
        buf[4 + 3 * i] = pixels[i][2]
        buf[4 + 3 * i + 1] = pixels[i][1]
        buf[4 + 3 * i + 2] = pixels[i][0]
    }
    sock.send(buf, 0, buf.length, PORT, HOST, function(err) {
        if (err)
            console.error(err.stack || err.message)
    })
}

var SimplexNoise = require('simplex-noise');
var simplex = new SimplexNoise(Math.random);

var start = Date.now()
let pixels = new Array(LEDS)
setInterval(function() {
    let now = Date.now()
    var i
    for(i = 0; i < 20; i += 1) {
        let t = (20 * (1 - (now % 2000) / 2000) + i) % 20
        pixels[i] = [Math.max(0, (t - 16) * 63), 0, 0]
    }

    let t = (Date.now() - start) / 10
    let w = 206
    for (i = 0; i < w; i++) {
        var h2_colorspread = 180; // 10..180
        var h2_width = 100; //40..120
        h2_colorspread = (Math.sin(t / 4223) * 0.5 + 0.5) * 170 + 10;
        h2_width = (Math.sin(t / 2342) * 0.5 + 0.5) * 40 + 20;
        var h = simplex.noise2D(i / 125, t / 2256) * 0.5 + 0.5;
        var h2 = simplex.noise2D(i / h2_width + t / 450, t / 467) * 0.5 + 0.5;
        var v = simplex.noise2D(i / w + t / 300, t / 368) * 0.5 + 0.5;
        var rgb = HSVtoRGB((h * 360) + h2 * h2_colorspread, 1, v / 3 + 0.66);
        // if (x==0) console.log({t: t, rgb: rgb, v:v, h:h, h2:h2 });

        pixels[20 + i] = [rgb.r, rgb.g, rgb.b]
    }

    send(pixels)
    
    t += 1
}, 1000 / FPS)

function HSVtoRGB(h, s, v) {
    // this may look a bit nerdy, but everything is just
    // according to http://en.wikipedia.org/wiki/HSL_and_HSV#From_HSV
    var c = v * s;
    var h_ = (h % 360) / 60;
    var x = c * (1 - Math.abs((h_ % 2) - 1));
    var rgb = [
        [c, x, 0],
        [x, c, 0],
        [0, c, x],
        [0, x, c],
        [x, 0, c],
        [c, 0, x]
    ][Math.floor(h_)]
    // catch undefined
    rgb = rgb ? rgb : [0, 0, 0];
    var m = v - c;
    return {
        'r': Math.round((rgb[0] + m) * 255),
            'g': Math.round((rgb[1] + m) * 255),
            'b': Math.round((rgb[2] + m) * 255)
    };
}
