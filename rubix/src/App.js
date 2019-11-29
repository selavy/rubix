import React from 'react';
import './App.css';
import * as THREE from "three";
import Cube from "cubejs";

class App extends React.Component {
    // constructor(props) {
    //     super(props);
    //     this.state = { solverInitialized: false };
    // }

    async componentDidMount() {
        // console.log("initializing cube solver...");
        // await Cube.initSolver();
        // console.log("finished initializing cube solver!");
        // this.setState({ solverInitialized: true });

        // {
        //     const desc = "DRLUUBFBRBLURRLRUBLRDDFDLFUFUFFDBRDUBRUFLLFDDBFLUBLRBD";
        //     const c = Cube.fromString(desc);
        //     const solution = c.solve();
        //     console.log("JS solution:");
        //     console.log(solution);
        // }

        var scene = new THREE.Scene();
        var camera = new THREE.PerspectiveCamera( 75, window.innerWidth/window.innerHeight, 0.1, 1000 );
        var renderer = new THREE.WebGLRenderer();
        renderer.setSize( window.innerWidth, window.innerHeight );
        // document.body.appendChild( renderer.domElement );
        // use ref as a mount point of the Three.js scene instead of the document.body
        this.mount.appendChild( renderer.domElement );
        var geometry = new THREE.BoxGeometry( 1, 1, 1 );
        var material = new THREE.MeshBasicMaterial( { color: 0x00ff00 } );
        var cube = new THREE.Mesh( geometry, material );
        scene.add( cube );
        camera.position.z = 5;
        var animate = function () {
            requestAnimationFrame( animate );
            cube.rotation.x += 0.01;
            cube.rotation.y += 0.01;
            renderer.render( scene, camera );
        };
        animate();
    }

    render() {
        const desc = "DRLUUBFBRBLURRLRUBLRDDFDLFUFUFFDBRDUBRUFLLFDDBFLUBLRBD";
        // const desc = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
        const cube = Cube.fromString(desc);
        console.log("BEFORE:");
        console.log(cube.isSolved());
        console.log(cube.asString());

        cube.move("D2 R' D' F2 B D R2 D2 R' F2 D' F2 U' B2 L2 U2 D R2 U");
        console.log("AFTER:");
        console.log(cube.asString());
        console.log(cube.isSolved());

        return (
            <div ref={ref => (this.mount = ref)} />
        );
    }
}

export default App;
