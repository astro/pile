module.exports = function(presets) {
    var star_ttl = 200 + 1800 * Math.random();
    var star_density = 0.0001 + 0.0009 * Math.random();
    var colorful = Math.random() < 0.3;
    var stars = [];

    return {
        render: function(output, cb) {
            var frame = [];
            for(var i = 0; i < output.x * output.y; i++) {
                frame.push([0, 0, 0]);
            }

            var now = Date.now();
            if (Math.random() < (output.x * output.y * star_density)) {
                stars.push({
                    x: Math.floor(output.x * Math.random()),
                    y: Math.floor(output.y * Math.random()),
                    birth: now,
                    r: 127 + 128 * Math.random(),
                    g: 127 + 128 * Math.random(),
                    b: 127 + 128 * Math.random()
                });
            }
            stars.forEach(function(star) {
                var i = star.y * output.x + star.x;
                star.health = 255 * (1 - (now - star.birth) / star_ttl);
                var brightness = Math.max(frame[i][0], star.health);
                if (colorful) {
                    frame[i] = [star.r * brightness, star.g * brightness, star.b * brightness];
                } else {
                    frame[i] = [brightness, brightness, brightness];
                }
            });
            stars = stars.filter(function(star) {
                return star.health > 0;
            });

            cb(null, frame);
        }
    };
};
