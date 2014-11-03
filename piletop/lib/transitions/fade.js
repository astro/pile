module.exports = function(opts) {
    return {
        render: function(output, from, to, cb) {
            var t = opts.getTime();
            var frame = [];
            for(var i = 0; i < output.x * output.y; i++) {
                var c1 = from[i];
                var c2 = to[i];
                function mix(i) {
                    return (1 - t) * c1[i] + t * c2[i];
                }
                var mixed = [mix(0), mix(1), mix(2)];
                frame.push(mixed);
            }

            cb(null, frame);
        }
    };
};
