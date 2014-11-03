var dgram = require('dgram');
var OPC = require("openpixelcontrol");
var Writable = require('stream').Writable;
var util = require('util');


util.inherits(OSCUDPOutput, Writable);
function OSCUDPOutput(config) {
    Writable.call(this, { objectMode: true, highWaterMark: 0 });

    this.sock = dgram.createSocket('udp6');
    this.sock.on('error', function(err) {
        this.emit('error', err);
    }.bind(this));

    this.x = config.x;
    this.y = config.y;
    this.host = config.host;
    this.port = config.port;
    this.channel = config.channel;
    this.interval = Math.ceil(1000 / config.fps);
    console.log(config.fps + " FPS: " + this.interval + " ms");
    this.nextSend = Date.now();
    console.log("OSCUDPOutput", config);

    this.statsTime = Date.now();
    this.statsPkts = 0;
};
module.exports = OSCUDPOutput;

OSCUDPOutput.prototype._write = function(framebuf, enc, cb) {
    var now = Date.now();
    // console.log("OSCUDPOutput._write in", this.nextSend - now, "ms");
    if (now < this.nextSend) {
        setTimeout(this._write.bind(this, framebuf, enc, cb), this.nextSend - now);
        return;
    }

    // console.log("OSC frame:", framebuf);
    var buf = OPC.messageToBuf({
        channel: this.channel,
        command: 0,
        data: framebuf
    });
    this.nextSend = now + this.interval;
    // console.log("send", buf.length, "bytes to", this.host+":"+this.port);
    this.sock.send(buf, 0, buf.length, this.port, this.host, function(err) {
        if (err) {
            console.error(err);
        }

        this.statsPkts++;
        var now = Date.now();
        if (this.statsTime + 1000 <= now) {
            console.log("Sent", Math.floor(1000 * this.statsPkts / (now - this.statsTime)), "pps");
            this.statsTime = now;
            this.statsPkts = 0;
        }
        cb(err);
    }.bind(this));
};
