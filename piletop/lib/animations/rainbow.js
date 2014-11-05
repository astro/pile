module.exports = function(presets) {
    var start = Date.now();
    var span = 90 + 450 * Math.random();
    var cycle = 1000 + 3000 * Math.random();
    if (Math.random() < 0.5) {
        cycle *= -1;
    }

    return {
        render: function(output, cb) {
            var frame = [];
            var t = (Date.now() - start) / cycle;
            for(var x = 0; x < output.x; x++) {
                var hue = span * (t + x / output.x);
                while(hue < 0) {
                    hue += 360;
                }
                var rgb = HSVtoRGB(hue, 1, 1);
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
