module.exports = function(presets) {
    var start = Date.now();
    var colors = presets.colors;
    var speed = 1000 + 4000 * Math.random();
    var offset = Math.floor(colors.length * Math.random());

    return {
        render: function(output, cb) {
            var now = Date.now();
            var t = (now - start) / speed;

            var c1 = colors[Math.floor(offset + t) % colors.length];
            var c2 = colors[(Math.floor(offset + t + 1)) % colors.length];
            var a = t - Math.floor(t);
            function mix(i) {
                return (1 - a) * c1[i] + a * c2[i];
            }
            var mixed = [mix(0), mix(1), mix(2)];
            // console.log("colorfade mixed", mixed);

            var frame = [];
            for(var i = 0; i < output.x * output.y; i++) {
                frame.push(mixed);
            }

            cb(null, frame);
        }
    };
};
