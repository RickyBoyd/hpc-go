use std::env;
use std::fs::File;
use std::io::prelude::*;

const NSPEEDS: usize = 9;
const FINALSTATEFILE: &'static str = "final_state.dat";
const AVVELSFILE: &'static str = "av_vels.dat";

/* struct to hold the parameter values */
struct Param {
    nx: usize,            /* no. of cells in x-direction */
    ny: usize,           /* no. of cells in y-direction */
    max_iters: usize,      /* no. of iterations */
    reynolds_dim: i32,  /* dimension for Reynolds number */
    density: f64,       /* density per link */
    accel: f64,         /* density redistribution */
    omega: f64,         /* relaxation parameter */
}

/* struct to hold the 'speed' values */

#[derive(Clone, Copy)]
struct Speed {
    speeds: [f64; NSPEEDS],
}

fn main() {
    println!("Hello rust!");

    let args: Vec<String> = env::args().collect();

    let paramfile = &args[1];
    let obstaclefile = &args[2];

    println!("Searching for {}", paramfile);
    println!("In file {}", obstaclefile);

    let (params, mut cells, mut tmp_cells, obstacles, mut av_vels) = initialise(paramfile, obstaclefile);
    println!("nx: {}", params.nx);

    for _ in 0..params.max_iters {
        timestep(&params, &mut cells, &mut tmp_cells, &obstacles);
        //av_vels[tt] = av_velocity(params, cells, obstacles);
        //printf("==timestep: %d==\n", tt);
        //printf("av velocity: %.12E\n", av_vels[tt]);
        //printf("tot density: %.12E\n", total_density(params, cells));
    }

    //gettimeofday(&timstr, NULL);
    //toc = timstr.tv_sec + (timstr.tv_usec / 1000000.0);
    //getrusage(RUSAGE_SELF, &ru);
    //timstr = ru.ru_utime;
    //usrtim = timstr.tv_sec + (timstr.tv_usec / 1000000.0);
    //timstr = ru.ru_stime;
    //systim = timstr.tv_sec + (timstr.tv_usec / 1000000.0);

    /* write final values and free memory */
    //printf("==done==\n");
    //printf("Reynolds number:\t\t%.12E\n", calc_reynolds(params, cells, obstacles));
    //printf("Elapsed time:\t\t\t%.6lf (s)\n", toc - tic);
    //printf("Elapsed user CPU time:\t\t%.6lf (s)\n", usrtim);
    //printf("Elapsed system CPU time:\t%.6lf (s)\n", systim);
    //write_values(params, cells, obstacles, av_vels);
}

fn timestep(params: &Param, cells: &mut Vec<Speed>, tmp_cells: &mut Vec<Speed>, obstacles: &Vec<bool>) {
    accelerate_flow(params, cells, obstacles);
    propagate(params, cells, tmp_cells);
    rebound(params, cells, tmp_cells, obstacles);
    //collision(params, cells, tmp_cells, obstacles);
}

fn accelerate_flow(params: &Param, cells: &mut Vec<Speed>, obstacles: &Vec<bool>) {
    /* compute weighting factors */
    let w1: f64 = params.density * params.accel / 9.0;
    let w2: f64 = params.density * params.accel / 36.0;

    /* modify the 2nd row of the grid */
    let ii = params.ny - 2;

    for jj in 0..params.nx {
        /* if the cell is not occupied and
        ** we don't send a negative density */
        if (!obstacles[ii * params.nx + jj]
            && (cells[ii * params.nx + jj].speeds[3] - w1) > 0.0
            && (cells[ii * params.nx + jj].speeds[6] - w2) > 0.0
            && (cells[ii * params.nx + jj].speeds[7] - w2) > 0.0) {
            /* increase 'east-side' densities */
            cells[ii * params.nx + jj].speeds[1] += w1;
            cells[ii * params.nx + jj].speeds[5] += w2;
            cells[ii * params.nx + jj].speeds[8] += w2;
            /* decrease 'west-side' densities */
            cells[ii * params.nx + jj].speeds[3] -= w1;
            cells[ii * params.nx + jj].speeds[6] -= w2;
            cells[ii * params.nx + jj].speeds[7] -= w2;
        }
    }
}

fn propagate(params: &Param, cells: &Vec<Speed>, tmp_cells: &mut Vec<Speed>) {
    /* loop over _all_ cells */
    for ii in 0..params.ny {
        for jj in 0..params.nx {
            /* determine indices of axis-direction neighbours
            ** respecting periodic boundary conditions (wrap around) */
            let y_n = (ii + 1) % params.ny;
            let x_e = (jj + 1) % params.nx;
            let y_s = if ii == 0 { ii + params.ny - 1 } else { ii - 1 };
            let x_w = if jj == 0 { jj + params.nx - 1 } else { jj - 1 };
            /* propagate densities to neighbouring cells, following
            ** appropriate directions of travel and writing into
            ** scratch space grid */
            tmp_cells[ii * params.nx + jj].speeds[0]  = cells[ii * params.nx + jj].speeds[0]; /* central cell, no movement */
            tmp_cells[ii * params.nx + x_e].speeds[1] = cells[ii * params.nx + jj].speeds[1]; /* east */
            tmp_cells[y_n * params.nx + jj].speeds[2]  = cells[ii * params.nx + jj].speeds[2]; /* north */
            tmp_cells[ii * params.nx + x_w].speeds[3] = cells[ii * params.nx + jj].speeds[3]; /* west */
            tmp_cells[y_s * params.nx + jj].speeds[4]  = cells[ii * params.nx + jj].speeds[4]; /* south */
            tmp_cells[y_n * params.nx + x_e].speeds[5] = cells[ii * params.nx + jj].speeds[5]; /* north-east */
            tmp_cells[y_n * params.nx + x_w].speeds[6] = cells[ii * params.nx + jj].speeds[6]; /* north-west */
            tmp_cells[y_s * params.nx + x_w].speeds[7] = cells[ii * params.nx + jj].speeds[7]; /* south-west */
            tmp_cells[y_s * params.nx + x_e].speeds[8] = cells[ii * params.nx + jj].speeds[8]; /* south-east */
        }
    }
}

fn rebound(params: &Param, cells: &mut Vec<Speed>, tmp_cells: &Vec<Speed>, obstacles: &Vec<bool>) {
    /* loop over the cells in the grid */
    for ii in 0..params.ny {
        for jj in 0..params.nx {
            /* if the cell contains an obstacle */
            if obstacles[ii * params.nx + jj] {
                /* called after propagate, so taking values from scratch space
                ** mirroring, and writing into main grid */
                cells[ii * params.nx + jj].speeds[1] = tmp_cells[ii * params.nx + jj].speeds[3];
                cells[ii * params.nx + jj].speeds[2] = tmp_cells[ii * params.nx + jj].speeds[4];
                cells[ii * params.nx + jj].speeds[3] = tmp_cells[ii * params.nx + jj].speeds[1];
                cells[ii * params.nx + jj].speeds[4] = tmp_cells[ii * params.nx + jj].speeds[2];
                cells[ii * params.nx + jj].speeds[5] = tmp_cells[ii * params.nx + jj].speeds[7];
                cells[ii * params.nx + jj].speeds[6] = tmp_cells[ii * params.nx + jj].speeds[8];
                cells[ii * params.nx + jj].speeds[7] = tmp_cells[ii * params.nx + jj].speeds[5];
                cells[ii * params.nx + jj].speeds[8] = tmp_cells[ii * params.nx + jj].speeds[6];
            }
        }
    }
}

fn initialise(paramfile: &str, obstaclefile: &str) -> (Param, Vec<Speed>, Vec<Speed>, Vec<bool>, Vec<f64>) {

    let mut f = File::open(paramfile).expect("could not open input parameter file");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");


    println!("{}", contents);
    let mut lines = contents.lines();

    let nx: usize = lines.next().unwrap().parse().unwrap();
    let ny: usize = lines.next().unwrap().parse().unwrap();
    let max_iters: usize = lines.next().unwrap().parse().unwrap();
    let reynolds_dim: i32 = lines.next().unwrap().parse().unwrap();
    let density: f64 = lines.next().unwrap().parse().unwrap();
    let accel: f64 = lines.next().unwrap().parse().unwrap();
    let omega: f64 = lines.next().unwrap().parse().unwrap();

    let params = Param{nx, ny, max_iters, reynolds_dim, density, accel, omega};


    // /* main grid */
    //let mut cells: Vec<Speed> = Vec::with_capacity(params.ny * params.nx);
    //let mut tmp_cells: Vec<Speed> = Vec::with_capacity(params.ny * params.nx);

    let mut cells: Vec<Speed> = vec![Speed{speeds: [0.0; NSPEEDS]}; params.ny * params.nx];
    let mut tmp_cells: Vec<Speed> = vec![Speed{speeds: [0.0; NSPEEDS]}; params.ny * params.nx];


    /* initialise densities */
    let w0 = params.density * 4.0 / 9.0;
    let w1 = params.density      / 9.0;
    let w2 = params.density      / 36.0;
    println! ("Here") ;
    for ii in 0..params.ny {
        for jj in 0..params.nx {
            /* centre */
            cells[ii * params.nx + jj].speeds[0] = w0;
            /* axis directions */
            cells[ii * params.nx + jj].speeds[1] = w1;
            cells[ii * params.nx + jj].speeds[2] = w1;
            cells[ii * params.nx + jj].speeds[3] = w1;
            cells[ii * params.nx + jj].speeds[4] = w1;
            /* diagonals */
            cells[ii * params.nx + jj].speeds[5] = w2;
            cells[ii * params.nx + jj].speeds[6] = w2;
            cells[ii * params.nx + jj].speeds[7] = w2;
            cells[ii * params.nx + jj].speeds[8] = w2;
        }
    }

    /* the map of obstacles */
    let mut obstacles = vec![false; (params.ny * params.nx)];

    /* open the obstacle data file */
    let mut f = File::open(obstaclefile).expect("could not open obstacle parameter file");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the obstacle file");

    let mut lines = contents.lines();
    for line in lines {
        let res: Vec<u16> = line.split(" ").map(|s| s.parse().unwrap()).collect();
        obstacles[res[1] as usize * params.nx + res[0] as usize] = true;
        println!("{} {} {}", res[0], res[1], res[2]);
    }

    /*
     ** allocate space to hold a record of the avarage velocities computed
     ** at each timestep
     */
    let mut av_vels = vec![0.0; params.max_iters];

    return (params, cells, tmp_cells, obstacles, av_vels);
}