var config = require("../config");
var fs = require('fs');
var Readable = require('stream').Readable;
var util = require('util');
var through = require('through2');

var animationsPath = __dirname + "/animations";
var animations = fs.readdirSync(animationsPath).filter(function(name) {
    return !/^pulse/.test(name)
}).map(function(name) {
    return {
        name: name, 
        module: require(animationsPath + "/" + name)
    };
// }).filter(function(a) {
//     return /^star/.test(a.name);
});

var transitionsPath = __dirname + "/transitions";
var transitions = fs.readdirSync(transitionsPath).map(function(name) {
    return require(transitionsPath + "/" + name);
});


function makeBufferizer(opts) {
    function clip(x) {
        return Math.floor(Math.max(0, Math.min(opts.max, x)));
    }
    opts.gamma = opts.gamma || 2.5;
    function gamma(x) {
        return 255 * (Math.pow(x / 255, opts.gamma));
    }
    
    return through.obj({ highWaterMark: 0 }, function(frame, enc, cb) {
        if (Buffer.isBuffer(frame)) {
            this.push(frame);
        } else {
            var buf = new Buffer(frame.length * 3);
            for(var i = 0; i < frame.length; i++) {
                for(var j = 0; j < 3; j++) {
                    buf[3 * i + j] = clip(gamma(frame[i][j]));
                }
            }
            this.push(buf);
            // console.log("bufferized", frame.slice(0, 4), "to", buf.slice(0, 12));
        }

        cb();
    });
}


function TransitionRender(opts) {
    Readable.call(this, { objectMode: true, highWaterMark: 0 });

    this.opts = opts;

    var start = Date.now();
    var getTime = function() {
        return Math.max(0, Math.min(1, (Date.now() - start) / opts.duration));
    };

    this.transition = opts.module({
        getTime: getTime
    });
}

TransitionRender.prototype.render = function(output, cb) {
    this.opts.from.render(output, function(err, from) {
        if (err) {
            this.emit('error', err);
            return;
        }

        this.opts.to.render(output, function(err, to) {
            if (err) {
                this.emit('error', err);
                return;
            }

            this.transition.render(output, from, to, cb);
        }.bind(this));
    }.bind(this));
};

/* Handles animations and transitions per output */
util.inherits(Director, Readable);
function Director(output) {
    Readable.call(this, { objectMode: true, highWaterMark: 0 });

    this.output = output;
    this.source = null;
}

Director.prototype._read = function() {
    // console.log("director _read", this.source);
    if (!this.source) {
        this.reading = true;
        return;
    }

    this.source.render(this.output, function(err, frame) {
        // console.log("source rendered", err, frame && frame.length);
        this.reading = false;
        if (err) {
            this.emit('error', err);
            return;
        }

        this.push(frame);
    }.bind(this));
};

Director.prototype.setNextSource = function(nextSource) {
    console.log("nextSource!");
    if (!this.source) {
        this.source = nextSource;
    } else {
        var prevSource = this.source;
        this.source = new TransitionRender({
            from: prevSource,
            to: nextSource,
            duration: config.transitionDuration,
            module: transitions[Math.floor(transitions.length * Math.random())]
        });
        setTimeout(function() {
            this.source = nextSource;
            if (prevSource && prevSource.close) {
                prevSource.close();
            }
            if (this.reading) {
                this._read();
            }
        }.bind(this), config.transitionDuration);
    }
    if (this.reading) {
        this._read();
    }
};


var directors = config.outputs.map(function(outputConfig) {
    var outputModule = require("./outputs/" + outputConfig.type);
    var output = new outputModule(outputConfig);
    var bufferize = makeBufferizer({
        gamma: outputConfig.gamma,
        max: outputConfig.max
    });
    var director = new Director(output);
    director.pipe(bufferize).pipe(output);
    return director;
});

/* Idle timer */
function zap() {
    var animation = animations[Math.floor(animations.length * Math.random())];
    console.log("zap to animation", animation.name);
    var nextSource = animation.module(config.animationPresets);

    directors.forEach(function(director) {
        director.setNextSource(nextSource);
    });

    setTimeout(zap, config.nextIdleAnimation);
}
zap();
