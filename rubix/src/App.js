import React from 'react';
import './App.css';
import * as THREE from "three";
import Cube from "cubejs";

const ORANGE = 0xFF6414;
const BLUE   = 0x2914FF;
const WHITE  = 0xFFFFFF;
const GREEN  = 0x0FCD1D;
const YELLOW = 0xE0FF2E;
const RED    = 0xFA1616;
const colors = {
    "U": WHITE,
    "R": BLUE,
    "F": YELLOW,
    "D": GREEN,
    "L": RED,
    "B": ORANGE,
};

class App extends React.Component {
    constructor(props) {
        super(props);
        const desc = "DRLUUBFBRBLURRLRUBLRDDFDLFUFUFFDBRDUBRUFLLFDDBFLUBLRBD";
        this.state = {
            startDesc: desc,
            moves: [],
            cube: Cube.fromString(desc),
            solverReady: false,
            solution: "D2 R' D' F2 B D R2 D2 R' F2 D' F2 U' B2 L2 U2 D R2 U".split(' '),
        };
    }

    async componentDidMount() {
        var camera = new THREE.PerspectiveCamera(
            75,                                     // FOV -- degrees
            window.innerWidth / window.innerHeight, // Aspect Ratio
            1  ,                                    // Near Clipping Plane
            10000,                                  // Far Clipping Plane
        );
        camera.position.z = 500;
        camera.position.x = 0;
        camera.position.y = 0;

        var scene = new THREE.Scene();
        scene.background = new THREE.Color(0xF0F0F0);

        const squareSize = 50;
        const spacerSize = 5;
        const faceSize = 3*squareSize + 8*spacerSize;
        const origin = { x: -300, y: -50, z: 0 };
        const coords = [
            { x: 1, y:  1, z: 0 }, // U
            { x: 2, y:  0, z: 0 }, // R
            { x: 1, y:  0, z: 0 }, // F
            { x: 1, y: -1, z: 0 }, // D
            { x: 0, y:  0, z: 0 }, // L
            { x: 3, y:  0, z: 0 }, // B
        ];
        const positions = coords.map(({x, y, z}) => {
            return {
                x: origin.x + x*faceSize,
                y: origin.y + y*faceSize,
                z: origin.z + z*faceSize,
            }
        });
        const offsets = [
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
            { x: 1, y: 1, z: 0 },
        ];

        const cube = this.state.cube;
        const desc = cube.asString();
        for (var faceIdx = 0; faceIdx < 6; faceIdx++) {
            for (var x = 0; x < 3; x++) {
                for (var y = 0; y < 3; y++) {
                    const i = 9*faceIdx + 3*y + x;
                    const pos = positions[faceIdx];
                    const off = offsets[faceIdx];
                    const color = colors[desc[i]];
                    const geo = new THREE.PlaneBufferGeometry(squareSize, squareSize);
                    const mat = new THREE.MeshBasicMaterial({
                        color: color,
                        side: THREE.FrontSide
                    });
                    const face = new THREE.Mesh(geo, mat);
                    face.position.x = pos.x + x*off.x*(squareSize + spacerSize);
                    face.position.y = pos.y + y*off.y*(squareSize + spacerSize);
                    face.position.z = pos.z + y*off.z*(squareSize + spacerSize);
                    scene.add(face);
                }
            }
        }


        var renderer = new THREE.WebGLRenderer();
        renderer.setSize(window.innerWidth - 200, window.innerHeight - 200);
        // document.body.appendChild(renderer.domElement);
        this.mount.appendChild(renderer.domElement);

        function animate() {
            requestAnimationFrame(animate);
            renderer.render(scene, camera);
        }
        animate();
    }

    render() {
        return (
            <div ref={ref => (this.mount = ref)} />
        );
    }
}

export default App;
