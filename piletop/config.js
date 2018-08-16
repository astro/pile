exports.outputs = [{
    type: 'osc_udp',
    host: '::ffff:172.22.99.206',
    port: 2342,
    channel: 254,
    x: 206,
    y: 1,
    fps: 90,
    gamma: 2.5,
    max: 255
}];

exports.nextIdleAnimation = 5 * 1000;
exports.transitionDuration = 1.2 * 1000;

exports.animationPresets = {
    colors: [
        [0, 255, 255],
        [200, 220, 240],
        [64, 64, 192],
        [0, 255, 192],
        [64, 64, 128],
        [128, 192, 240],
        [0, 64, 255],
        [0, 192, 255],
        [64, 64, 64],
        [0, 255, 64]
    ]
};
