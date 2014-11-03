module.exports = function(presets) {
    var stars = [];

    return {
        render: function(output, cb) {
            var frame = [];
            for(var i = 0; i < output.x * output.y; i++) {
                frame.push([0, 0, 0]);
            }

            var now = Date.now();
            if (Math.random() < (output.x * output.y / 1000)) {
                stars.push({
                    x: Math.floor(output.x * Math.random()),
                    y: Math.floor(output.y * Math.random()),
                    birth: now
                });
            }
            stars.forEach(function(star) {
                var i = star.y * output.x + star.x;
                star.health = 255 * (1 - (now - star.birth) / 500);
                var brightness = Math.max(frame[i][0], star.health);
                frame[i] = [brightness, brightness, brightness];
            });
            stars = stars.filter(function(star) {
                return star.health > 0;
            });

            cb(null, frame);
        }
    };
};
