module.exports = function(presets) {
    var star_ttl = 200 + 1800 * Math.random();
    var star_density = 0.0001 + 0.0009 * Math.random();
    var colorful = Math.random() < 0.3;
    var stars = [];

    console.log("Stars", { ttl: star_ttl, density: star_density, colorful: colorful })

    return {
        render: function(output, cb) {
            var frame = [];
            for(var i = 0; i < output.x * output.y; i++) {
                frame.push([0, 0, 0]);
            }

            var now = Date.now();
            if (stars.length < 1 || Math.random() < (output.x * output.y * star_density)) {
                console.log("new star, " + (stars.length + 1) + " stars")
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
                star.health = 1 - (now - star.birth) / star_ttl;
                // if (arguments[1] == 0) 
                // console.log("star", arguments[1], {age: now - star.birth,health: star.health,i:i})
                var brightness = Math.max(0, 255 * star.health);
                if (colorful) {
                    frame[i] = maxRGB(frame[i], [star.r * brightness, star.g * brightness, star.b * brightness]);
                } else {
                    frame[i] = maxRGB(frame[i], [brightness, brightness, brightness]);
                    // console.log("frame",i,frame[i])
                }
            });
            stars = stars.filter(function(star) {
                if (star.health <= 0)
                  console.log("del star, " + (stars.length - 1) + " stars")
                return star.health > 0;
            });

            cb(null, frame);
        }
    };
};

function maxRGB(a, b) {
    return [
        Math.max(a[0], b[0]),
        Math.max(a[1], b[1]),
        Math.max(a[2], b[2])
    ]
}
