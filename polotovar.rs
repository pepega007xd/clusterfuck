extern crate core;
use core::ptr;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::env;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Obj {
    id: i32,
    x: f32,
    y: f32,
}

impl Obj {
    fn dist(&self, other: &Obj) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;

        (dx * dx + dy * dy).sqrt()
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Cluster {
    size: i32,
    cap: i32,
    obj: *mut Obj,
}

impl Cluster {
    fn dist(&mut self, other: &mut Cluster) -> f32 {
        // very precise approximation of 1000 * sqrt(2)
        let mut min = 2000.0;
        let mut dist;

        let (v1, v2) = (
            c2v(self),
            c2v(other)
        );

        for o1 in v1.iter() {
            for o2 in v2.iter() {
                dist = o1.dist(o2);
                if dist < min {min = dist}
            }
        }

		mem::forget(v1);
		mem::forget(v2);
        
        min
    }
}

fn c2v(c: *mut Cluster) -> Vec<Obj> {
    unsafe {
        let c = &mut *c;
        Vec::from_raw_parts(c.obj, c.size as usize, c.cap as usize)
    }
}

fn v2c(mut v: Vec<Obj>, c: *mut Cluster) {
    unsafe {
        let c = &mut *c;
        c.size = v.len() as i32;
        c.cap = v.capacity() as i32;
        c.obj = v.as_mut_ptr();
    }
	mem::forget(v);
}

fn carr2v(carr: *mut Cluster, narr: i32) -> Vec<Cluster> {
    unsafe {
        Vec::from_raw_parts(carr, narr as usize, narr as usize)
    }
}

fn strlen(str: &*mut u8) -> usize {
    let mut count = 0;
    while unsafe {*(str.offset(count as isize)) != 0}  {
        count += 1;
    }

    count
}

fn print(c: *mut Cluster, count: i32) {
    println!("Clusters:");
    for i in 0..(count as isize) {
        unsafe {
            let len = (*c.offset(i)).size;
            print!("cluster {}:", i);
            for j in 0..(len as isize) {
                let id = (*(*c.offset(i)).obj.offset(j)).id;
                let x = (*(*c.offset(i)).obj.offset(j)).x;
                let y = (*(*c.offset(i)).obj.offset(j)).y;
                print!(" {}[{},{}]", id, x, y);
            }
            println!("");
        }
    }
    
    std::io::stdout().flush().unwrap();
}

#[no_mangle]
pub extern "C" fn init_cluster(c: *mut Cluster, cap: i32) {
    unsafe {
        let c = &mut *c;
        c.size = 0;
        // this allocates memory
        let mut vec = Vec::with_capacity(cap as usize);
        c.obj = vec.as_mut_ptr();
        c.cap = cap;
		mem::forget(vec);
    }
}

#[no_mangle]
pub extern "C" fn clear_cluster(c: *mut Cluster) {
    // this deallocates memory
    c2v(c);

    unsafe {
        let c = &mut *c;
        c.size = 0;
        c.cap = 0;
        c.obj = ptr::null_mut();
    }
}


#[no_mangle]
pub extern "C" fn append_cluster(c: *mut Cluster, obj: Obj) {
    let mut v = c2v(c);
    v.push(obj);
    v2c(v, c);
}

#[no_mangle]
pub extern "C" fn merge_clusters(c1: *mut Cluster, c2: *mut Cluster) {
    let mut v1 = c2v(c1);
    let mut v2 = c2v(c2);

    v1.append(&mut v2);
    v1.sort_by(|a, b| a.id.cmp(&b.id));

    v2c(v1, c1);
    v2c(v2, c2);
}

#[no_mangle]
pub extern "C" fn remove_cluster(carr: *mut Cluster, narr: i32, idx: i32) -> i32 {
    let mut varr = carr2v(carr, narr);
    varr.remove(idx as usize);
	mem::forget(varr);
    narr -1
}

#[no_mangle]
pub extern "C" fn obj_distance(o1: *mut Obj, o2: *mut Obj) -> f32 {
    unsafe {(&*o1).dist(&*o2)}
}

#[no_mangle]
pub extern "C" fn cluster_distance(c1: *mut Cluster, c2: *mut Cluster) -> f32 {
    unsafe {(&mut *c1).dist(&mut *c2)}
}

#[no_mangle]
pub extern "C" fn find_neighbors(carr: *mut Cluster, narr: i32, c1: *mut i32, c2: *mut i32) {
    let mut varr = carr2v(carr, narr);

    // this is a bit too much even for me
    let mut varr2 = varr.clone();

    let mut min = 69420.0;
    for (i, cl1) in varr.iter_mut().enumerate() {
        for (j, cl2) in varr2.iter_mut().enumerate().filter(|(j, _)| i < *j) {
            let dist = cl1.dist(cl2);
            if dist < min {
                min = dist;
                unsafe {*c1 = i as i32; *c2 = j as i32;}
            }
        }
    }
	mem::forget(varr);
} 

#[no_mangle]
pub extern "C" fn load_clusters(filename: *mut u8, carr: *mut *mut Cluster) -> i32 {
    let filename = unsafe {String::from_raw_parts(filename, strlen(&filename), strlen(&filename))};
    let fn2 = filename.clone();

    // this could've been a great shooting in the foot
    mem::forget(filename);
    
    let mut file = File::open(fn2).unwrap();

    let mut file_vnitrnosti = String::new();
    file.read_to_string(&mut file_vnitrnosti).unwrap();
    
    let count = file_vnitrnosti.split("\n")
        .next().unwrap()
        .split("=")
        .nth(1).unwrap()
        .parse().unwrap();

    let mut clusters = Vec::with_capacity(count);
    unsafe {*carr = clusters.as_mut_ptr();}

    file_vnitrnosti.split("\n")
        .skip(1)
        .for_each(|line| {
            let values = line.split_whitespace()
                .collect::<Vec<_>>();
            if values.len() < 3 {return;}
            let obj = Obj {
                id: values[0].parse().unwrap(),
                x: values[1].parse().unwrap(),
                y: values[2].parse().unwrap(),
            };

            let mut obj = vec![obj];

            clusters.push(Cluster {
                obj: obj.as_mut_ptr(),
                size: 1,
                cap: 1,
            });

			mem::forget(obj);
        });

	mem::forget(clusters);
    count as i32
}

#[no_mangle]
pub extern "C" fn mainfunc(argc: i32, argv: *mut *mut u8) -> i32 {
    if argc == 1 {
        println!("gimme more args or get tf outta here");
        return 69;
    }

    let mut clusters: *mut Cluster = 0 as *mut Cluster;
    let mut count = unsafe {load_clusters(*argv.offset(1), &mut clusters)};
    let final_count = env::args().nth(2).unwrap_or("1".to_string()).parse().unwrap();

    while count > final_count {
        let mut cl1 = 0;
        let mut cl2 = 0;
        find_neighbors(clusters, count, &mut cl1, &mut cl2);
        unsafe {merge_clusters(clusters.offset(cl1 as isize), clusters.offset(cl2 as isize))};
        count = remove_cluster(clusters, count, cl2);
    }

    print(clusters, count);

    0
}
