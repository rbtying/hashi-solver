import React from "react";
import Tesseract from "tesseract.js";
import cv from "@techstark/opencv-js";
import { solve } from "hashi-solver-wasm";
import "./style.css";

window.cv = cv;

class TestPage extends React.Component {
  constructor(props) {
    super(props);
    this.inputImgRef = React.createRef();
    this.dstImgRef = React.createRef();
    this.tmpImgRef = React.createRef();
    let initialized = false;

    try {
      const v = new cv.Mat();
      v.delete();
      initialized = true;
    } catch (e) {}

    this.state = {
      imgUrl: process.env.PUBLIC_URL + "/example.png",
      initialized,
      text: "",
      solving: false,
      soln: "",
    };

    cv["onRuntimeInitialized"] = () => {
      this.setState({ ...this.state, initialized: true });
    };
  }

  componentDidMount() {
    this.worker = Tesseract.createWorker({
      logger: (m) => {},
    });
  }

  /////////////////////////////////////////
  //
  // process image with opencv.js
  //
  /////////////////////////////////////////
  async processImage(imgSrc) {
    await this.worker.load();
    await this.worker.loadLanguage("eng");
    await this.worker.initialize("eng");
    const img = cv.imread(imgSrc);
    const imgGray = new cv.Mat();
    cv.cvtColor(img, imgGray, cv.COLOR_RGBA2GRAY);

    const thresholded = new cv.Mat();
    cv.threshold(imgGray, thresholded, 200, 255, cv.THRESH_BINARY_INV);

    let contours = new cv.MatVector();
    let hierarchy = new cv.Mat();
    cv.findContours(
      thresholded,
      contours,
      hierarchy,
      cv.RETR_TREE,
      cv.CHAIN_APPROX_SIMPLE
    );
    // draw contours with random Scalar
    const dst = cv.Mat.zeros(img.rows, img.cols, cv.CV_8UC1);

    for (let i = 0; i < contours.size(); ++i) {
      const rect = cv.boundingRect(contours.get(i));
      const area = rect.width * rect.height;
      const margin = 3;
      if (
        rect.width > 10 &&
        rect.height > 10 &&
        area > 100 &&
        area < (img.rows * img.cols) / 10
      ) {
        cv.rectangle(
          dst,
          new cv.Point(rect.x + margin, rect.y + margin),
          new cv.Point(
            rect.x + rect.width - 2 * margin,
            rect.y + rect.height - 2 * margin
          ),
          new cv.Scalar(255),
          cv.FILLED
        );
      }
    }

    let contours2 = new cv.MatVector();
    let hierarchy2 = new cv.Mat();
    // You can try more different parameters
    cv.findContours(
      dst,
      contours2,
      hierarchy2,
      cv.RETR_TREE,
      cv.CHAIN_APPROX_SIMPLE
    );

    const rectangles = [];
    const indices = [];

    const dst2 = cv.Mat.zeros(img.rows, img.cols, cv.CV_8UC4);
    for (let i = 0; i < contours2.size(); ++i) {
      const rect = cv.boundingRect(contours2.get(i));
      indices.push(i);
      rectangles.push(rect);
      // cv.rectangle(
      //   dst2,
      //   new cv.Point(rect.x, rect.y),
      //   new cv.Point(rect.x + rect.width, rect.y + rect.height),
      //   new cv.Scalar(255),
      //   1
      // );
    }

    cv.imshow(this.dstImgRef.current, dst2);

    const overlap = (r1, r2) => {
      return !(
        r1.x + r1.width < r2.x ||
        r2.x + r2.width < r1.x ||
        r1.y + r1.height < r2.y ||
        r2.y + r2.height < r1.y
      );
    };

    const values = [];
    for (let i = 0; i < rectangles.length; ++i) {
      const rect = rectangles[i];
      // if the rectangle overlaps with a preexisting rectangle, skip it
      let skip = false;
      for (let j = 0; j < i; ++j) {
        const rect2 = rectangles[j];
        if (overlap(rect, rect2)) {
          console.log("skipping " + i, rect, rect2);
          skip = true;
        }
      }
      if (!skip) {
        const smaller = img.roi(rect);
        smaller.copyTo(dst2.roi(rect));

        cv.imshow(this.dstImgRef.current, dst2);
        const x = rect.x;
        const y = rect.y;

        cv.imshow(this.tmpImgRef.current, smaller);
        const dataURL = this.tmpImgRef.current.toDataURL("image/png");
        const text = await this.worker.recognize(dataURL);
        const v = (text.data.text.match(/\d+/) || [])[0] || "?";
        values.push({ x, y, v });
        smaller.delete();
      }
    }

    let uniq_x = [];
    let uniq_y = [];
    for (let idx in values) {
      const x = values[idx].x;
      const y = values[idx].y;
      if (uniq_x.indexOf(x) === -1) {
        uniq_x.push(x);
      }
      if (uniq_y.indexOf(y) === -1) {
        uniq_y.push(y);
      }
    }
    uniq_x.sort((a, b) => a - b);
    uniq_y.sort((a, b) => a - b);

    let min_delta_x = Number.MAX_VALUE;
    for (let i = 1; i < uniq_x.length; ++i) {
      const delta = uniq_x[i] - uniq_x[i - 1];
      min_delta_x = Math.min(delta, min_delta_x);
    }

    let min_delta_y = Number.MAX_VALUE;
    for (let i = 1; i < uniq_y.length; ++i) {
      const delta = uniq_y[i] - uniq_y[i - 1];
      min_delta_y = Math.min(delta, min_delta_y);
    }

    let xs = [uniq_x[0]];
    for (let i = 1; i < uniq_x.length; ++i) {
      const delta = uniq_x[i] - uniq_x[i - 1];
      let num_steps = Math.round(delta / min_delta_x);
      while (num_steps > 1) {
        xs.push(-1);
        num_steps--;
      }
      xs.push(uniq_x[i]);
    }

    let ys = [uniq_y[0]];
    for (let i = 1; i < uniq_y.length; ++i) {
      const delta = uniq_y[i] - uniq_y[i - 1];
      let num_steps = Math.round(delta / min_delta_y);
      while (num_steps > 1) {
        ys.push(-1);
        num_steps--;
      }
      ys.push(uniq_y[i]);
    }

    let arr = [];
    for (let i = 0; i < ys.length; ++i) {
      arr[i] = [];
      for (let j = 0; j < xs.length; ++j) {
        arr[i][j] = " ";
      }
    }

    for (let idx in values) {
      const v = values[idx];
      const x = xs.indexOf(v.x);
      const y = ys.indexOf(v.y);
      arr[y][x] = v.v;
    }

    const str = arr.map((r) => r.join("")).join("\n");

    // need to release them manually
    img.delete();
    imgGray.delete();
    thresholded.delete();
    contours.delete();
    hierarchy.delete();
    dst.delete();
    this.setState({
      ...this.state,
      text: str,
    });
  }

  runSolve() {
    if (this.state.solving) {
      return;
    }
    this.setState({ ...this.state, solving: true });

    const soln = solve(this.state.text, 3);

    this.setState({ ...this.state, soln, solving: false });
  }

  render() {
    const { imgUrl, initialized } = this.state;
    return (
      <div>
        <div style={{ marginTop: "30px" }}>
          <span style={{ marginRight: "10px" }}>Select an image file:</span>
          <input
            type="file"
            name="file"
            accept="image/*"
            onChange={(e) => {
              if (e.target.files[0]) {
                this.setState({
                  imgUrl: URL.createObjectURL(e.target.files[0]),
                  initialized,
                });
              }
            }}
          />
          <span>
            from puzzle-bridges.com, select "Share", and then get the "progress
            screenshot"
          </span>
        </div>
        {!initialized && <p>Not initialized!</p>}

        {imgUrl && initialized && (
          <div className="images-container">
            <div className="image-card">
              <div style={{ margin: "10px" }}>↓↓↓ The original image ↓↓↓</div>
              <img
                alt="Original input"
                src={imgUrl}
                style={{ display: "none" }}
                onLoad={(e) => {
                  this.processImage(e.target);
                }}
              />
              <img
                alt="Original input"
                src={imgUrl}
                style={{ maxHeight: "450px", maxWidth: "450px" }}
              />
            </div>

            <div className="image-card">
              <div style={{ margin: "10px" }}>↓↓↓ dst Result ↓↓↓</div>
              <canvas
                ref={this.dstImgRef}
                style={{ maxHeight: "450px", maxWidth: "450px" }}
              />
            </div>
            <div className="image-card">
              <div style={{ margin: "10px" }}>↓↓↓ Recognized text ↓↓↓</div>
              <pre>
                <textarea
                  value={this.state.text}
                  rows={30}
                  cols={30}
                  onChange={(evt) =>
                    this.setState({ ...this.state, text: evt.target.value })
                  }
                />
              </pre>
              <canvas ref={this.tmpImgRef} style={{ display: "none" }} />
              {this.state.text.indexOf("?") >= 0 && (
                <p>
                  The <code>?</code> are most likely <code>8</code>s; the
                  recognition isn't perfect on those. Click{" "}
                  <button
                    onClick={(_) => {
                      this.setState({
                        ...this.state,
                        text: this.state.text.replace("?", "8"),
                      });
                    }}
                  >
                    here
                  </button>{" "}
                  to replace <code>?</code> with <code>8</code>
                </p>
              )}
            </div>
            <div className="image-card">
              <div style={{ margin: "10px" }}>↓↓↓ Solution ↓↓↓</div>
              <button onClick={() => this.runSolve()}>Try to solve</button>
              {this.state.solving && <p>...</p>}
              <pre>{this.state.soln}</pre>
            </div>
          </div>
        )}
      </div>
    );
  }
}

export default TestPage;
