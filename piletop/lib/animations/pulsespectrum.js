var pulsefreqs = require("../pulsefreqs");
var through = require('through2');

var freqs;
var maxIntensity = 1000;
var maxFreq = 3200;
pulsefreqs(function(err, src) {
    console.log("pulse src");
    src.pipe(through.obj(function(freqs_, enc, cb) {
        freqs = freqs_;
        for(var i = 0; i < freqs.length; i++) {
            var intensity = Math.log(freqs[i][1]);
            if (intensity > maxIntensity) {
                maxIntensity = intensity;
                console.log("maxIntensity", maxIntensity);
            }
        }
        cb();
    }));
});

module.exports = function(presets) {
    return {
        render: function(output, cb) {
            var frame = [];
            for(var x = 0; x < output.x; x++) {
                var hue = 360 * x / output.x;
                var val = 0;
                var freq1 = maxFreq * x / output.x;
                var freq2 = maxFreq * (x + 1) / output.x;
                var intensity = 0;
                for(var i = 0; freqs && i < freqs.length; i++) {
                    if (freqs[i][0] >= freq1 && freqs[i][0] < freq2) {
                        val = Math.max(intensity, freqs[i][1] / maxIntensity);
                    }
                    // console.log({ x: x, f1: freq1, f2: freq2, v: val });
                }
                var rgb = HSVtoRGB(hue, 1, val);
                var color = [rgb.r, rgb.g, rgb.b];
                for(var y = 0; y < output.y; y++) {
                    frame.push(color);
                }
            }

            cb(null, frame);
        }
    };
};

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
