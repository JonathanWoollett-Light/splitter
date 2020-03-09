extern crate image;
use std::path::Path;
use itertools::izip;
use std::env;
use image::{ImageBuffer, Rgb,Luma};
use image::imageops::FilterType;
use std::time::Instant;
use std::collections::VecDeque;
// Overall O'notation of 4n(ish) (n being image size=width*height)
// I think that's pretty good.

const B_SPACING:usize = 20usize; // Border space
// Maximum number of intial symbols that can be identified (larger images and more complex symbols require a higher number)
const MAX_SYMBOLS:usize = 1000usize;
const WHITE_SPACE_SYMBOL:char = ' '; // What symbol to use when priting white pixels
const LUMA_BOUNDARY:u8 = 135u8; // Luma less than set to 0 and more than set to 255.


// O(3n + some shit)
fn main() {
    let args: Vec<String> = env::args().collect();
    let img_name = &args[1];

    println!("img_name: {}",img_name);

    let path = &format!("images/{}",img_name);
    let path = Path::new(path);

    let img = image::open(path).unwrap().to_luma();
    
    let dims = img.dimensions();
    let (width,height) = (dims.0 as usize,dims.1 as usize);
    
    let mut img_raw:Vec<u8> = img.clone().into_raw();
    
    // 2d vector of size of image, where each pixel will be labelled as to which symbol it belongs
    let mut symbols:Vec<Vec<u32>> = to_bin2dvec(width,height,&mut img_raw);

    // Debug stuff to check binarisation worked:
    let check_img:ImageBuffer<Luma<u8>,Vec<u8>> = ImageBuffer::<Luma<u8>,Vec<u8>>::from_raw(width as u32,height as u32,img_raw).unwrap();
    check_img.save("check_img.png").unwrap();

    // Printing can be nice to visualize process.
    // But for larger images it simply prints useless spam in the console.
    if width <= 200 && height <= 400 {
        symbols_intial_prt(&symbols);
    }
    sweep_segmentation(&path,width,height,&mut symbols);
    //flood_segmentation(&path,width,height,&mut symbols);
}

#[allow(dead_code,non_snake_case)]
fn prt_u8_vec__as_2d((width,height):(usize,usize),vec:&Vec<u8>) -> () {
    println!();
    let shape = (width,height); // shape[0],shape[1]=row,column
    let spacing = 4*shape.0;
    println!("┌ {:<1$}┐","─",spacing);
    for row in 0..shape.1 {
        print!("│ ");
        for col in 0..shape.0 {
            print!("{} ",vec[row*width+col]);
            
        }
        println!("│");
    }
    println!("└ {:<1$}┘","─",spacing);
    print!("{:<1$}","",(spacing/2)-1);
    println!("[{},{}]",shape.0,shape.1);
    println!();
}

fn flood_segmentation(path:&Path,width:usize,height:usize,symbols:&mut Vec<Vec<u32>>) {
    let start = Instant::now();
    let mut symbol_count = 2u32;
    let mut pixels_in_symbols:Vec<Vec<(usize,usize)>> = Vec::new();
    let start_flood = Instant::now();
    for y in 0..height {
        for x in 0..width {
            if symbols[y][x] == 0 {
                pixels_in_symbols.push(Vec::new());
                let last_index = pixels_in_symbols.len()-1;
                //flood_fill_recursive(symbols,symbol_count,width,height,x,y,&mut pixels_in_symbols[last_index]);
                flood_fill_queue(symbols,symbol_count,width,height,x,y,&mut pixels_in_symbols[last_index]);
                symbol_count += 1;
            }
        }
    }
    println!("{} : Flood finished",time(start_flood));
    if width <= 200 && height <= 400 {
        symbols_classified_prt(&symbols);
    }

    // Set borders of symbols
    let borders = set_borders(symbols,&pixels_in_symbols,width,height,path);
    // Create symbol images
    let mut symbol_images:Vec<ImageBuffer<Luma<u8>,Vec<u8>>> = create_symbol_images(&pixels_in_symbols,&borders,width,height);
    // Export symbol images
    for i in 0..symbol_images.len() {
        let path = format!("split/{}.png",i);
        let mut scaled_image = image::imageops::resize(&mut symbol_images[i],35,35,FilterType::Triangle);
        image::imageops::colorops::invert(&mut scaled_image);
        scaled_image.save(path).unwrap();
    }
    println!("{} : Flood segmented",time(start));

    fn flood_fill_queue(symbols:&mut Vec<Vec<u32>>,symbol_count:u32,width:usize,height:usize,x:usize,y:usize,pixels:&mut Vec<(usize,usize)>) {
        pixels.push((x,y));
        symbols[y][x] = symbol_count;
        let mut queue:VecDeque<(usize,usize)> = VecDeque::new();
        queue.push_back((x,y));
        loop {
            if let Some(n) = queue.pop_front() {
                // +x
                if n.0 < width-1 {
                    let (x,y) = (n.0+1,n.1);
                    if symbols[y][x] == 0 {
                        pixels.push((x,y));
                        symbols[y][x] = symbol_count;
                        queue.push_back((x,y));
                    }
                }
                // -x
                if n.0 > 0 {
                    let (x,y) = (n.0-1,n.1);
                    if symbols[y][x] == 0 {
                        pixels.push((x,y));
                        symbols[y][x] = symbol_count;
                        queue.push_back((x,y));
                    }
                }
                // +y
                if n.1 < height-1 {
                    let (x,y) = (n.0,n.1+1);
                    if symbols[y][x] == 0 {
                        pixels.push((x,y));
                        symbols[y][x] = symbol_count;
                        queue.push_back((x,y));
                    }
                }
                // -y
                if n.1 > 0 {
                    let (x,y) = (n.0,n.1-1);
                    if symbols[y][x] == 0 {
                        pixels.push((x,y));
                        symbols[y][x] = symbol_count;
                        queue.push_back((x,y));
                    }
                }
            }
            else { break; } // If queue empty break
        }
    }
    fn set_borders(symbols:&mut Vec<Vec<u32>>,pixel_symbols:&Vec<Vec<(usize,usize)>>,width:usize,height:usize,path:&Path) -> Vec<((usize,usize),(usize,usize))> {
        let start = Instant::now();
        // Gets bounds
        let mut border_bounds:Vec<((usize,usize),(usize,usize))> = Vec::new();
        for symbol in pixel_symbols {
            let mut lower_x = symbol.iter().fold(width, |min,x| (if x.0 < min { x.0 } else { min }));
            let mut lower_y = symbol.iter().fold(height, |min,x| (if x.1 < min { x.1 } else { min }));
            let mut upper_x = symbol.iter().fold(0usize, |max,x| (if x.0 > max { x.0 } else { max }));
            let mut upper_y = symbol.iter().fold(0usize, |max,x| (if x.1 > max { x.1 } else { max }));

            if lower_x >= B_SPACING { lower_x -= B_SPACING; };
            if lower_y >= B_SPACING { lower_y -= B_SPACING; };
            if upper_x + B_SPACING < width { upper_x += B_SPACING; };
            if upper_y + B_SPACING < height { upper_y += B_SPACING; };

            border_bounds.push(((lower_x,lower_y),(upper_x,upper_y)));
        }
        // Copies image
        let mut border_img = image::open(path).unwrap().into_rgb();
        for (x,y,pixel) in border_img.enumerate_pixels_mut() {
            let val = if symbols[y as usize][x as usize] == 1 { 255 } else { 0 };
            *pixel = image::Rgb([val,val,val]);
        }

        // Sets borders
        let border_pixel = image::Rgb([255,0,0]); // Pixel to use as border
        for symbol in border_bounds.iter() {
            let min_x = (symbol.0).0;
            let min_y = (symbol.0).1;
            let max_x = (symbol.1).0;
            let max_y = (symbol.1).1;
            // Sets horizontal borders
            for i in min_x..max_x {
                *border_img.get_pixel_mut(i as u32,min_y as u32) = border_pixel;
                *border_img.get_pixel_mut(i as u32,max_y as u32) = border_pixel;
            }
            // Sets vertical borders
            for i in min_y..max_y {
                *border_img.get_pixel_mut(min_x as u32,i as u32) = border_pixel;
                *border_img.get_pixel_mut(max_x as u32,i as u32) = border_pixel;
            }
            // Sets bottom corner border
            *border_img.get_pixel_mut(max_x as u32,max_y as u32) = border_pixel;
        }
        border_img.save("borders.png").unwrap();
        println!("{} : Borders set",time(start));
        return border_bounds;
    }
    fn create_symbol_images(pixels_in_symbols:&Vec<Vec<(usize,usize)>>,borders:&Vec<((usize,usize),(usize,usize))>,width:usize,height:usize) -> Vec<ImageBuffer<Luma<u8>,Vec<u8>>> {
        let start = Instant::now();
        let sizes:Vec<(usize,usize)> = borders.iter().map(|lims| ((lims.1).0-(lims.0).0+1,(lims.1).1-(lims.0).1+1)).collect();
        // Default constructs to all black pixels, thus we set symbol pixels to white in following loop
        // TODO Look into constructing with default white pixels and drawing black pixels
        let mut symbol_images:Vec<ImageBuffer<Luma<u8>,Vec<u8>>> = sizes.iter().map(|size| ImageBuffer::<Luma<u8>,Vec<u8>>::new(size.0 as u32,size.1 as u32)).collect();
        // O(n)
        // Draws symbol images
        for i in 0..pixels_in_symbols.len() {
            let offset:(usize,usize) = borders[i].0;
            for pixel in pixels_in_symbols[i].iter() {
                let x = pixel.0;
                let y = pixel.1;
                let out_pixel = symbol_images[i].get_pixel_mut((x-offset.0) as u32,(y-offset.1) as u32);
                *out_pixel = image::Luma([255]);
            }
        }
        println!("{} : Created symbol images",time(start));
        return symbol_images;
    }
}

fn sweep_segmentation(path:&Path,width:usize,height:usize,symbols:&mut Vec<Vec<u32>>) {
    let start = Instant::now();
    // Index [i][t] represents whether symbol i and symbol t link
    let mut same_symbols:Vec<Vec<bool>> = vec!(vec!(false;MAX_SYMBOLS);MAX_SYMBOLS);
    let symbol_count = get_intial_symbols(height,width,symbols,&mut same_symbols);
    let mut links = get_links(&same_symbols);
    let change_symbols = symbol_changes(symbol_count,&mut links);
    let (consecutive_symbols,counter) = make_symbols_consectutive(symbol_count,&change_symbols);
    let pixels_in_symbols:Vec<Vec<(usize,usize)>> = unify_symbols(width,height,counter,symbols,&change_symbols,&consecutive_symbols);

    if width <= 100 && height <= 200 {
        symbols_classified_prt(&symbols);
    }

    // Set borders of symbols
    let borders:Vec<((usize,usize),(usize,usize))> = set_borders(width,height,path,&symbols,&pixels_in_symbols);
    
    // Create symbol images
    let mut symbol_images:Vec<ImageBuffer<Luma<u8>,Vec<u8>>> = create_symbol_images(width,height,&symbols,&borders);

    // Export symbol images
    for i in 0..symbol_images.len() {
        let path = format!("split/{}.png",i);
        let mut scaled_image = image::imageops::resize(&mut symbol_images[i],35,35,FilterType::Triangle);
        image::imageops::colorops::invert(&mut scaled_image);
        scaled_image.save(path).unwrap();
    }

    println!("{} : Sweep segmented",time(start));

    fn get_intial_symbols(height:usize,width:usize,symbols:&mut Vec<Vec<u32>>,same_symbols:&mut Vec<Vec<bool>>) -> u32 {
        let start = Instant::now();
        let mut symbol_count:u32 = 2;
        for y in 1..height-1 {
            for x in 1..width-1 {
                if symbols[x][y] == 0 {
                    if symbols[y][x-1] != 1 {
                        symbols[y][x] = symbols[y][x-1];
                    }
                    if symbols[y-1][x-1] != 1 {
                        // If adjacent pixel symbol:
                        //  If we have set pixel to symbol already: set similarity,
                        //  else set pixel to symbol
                        if symbols[y][x] != 0 {
                            same_symbols[symbols[y][x] as usize][symbols[y-1][x-1] as usize] = true;
                        } 
                        else { symbols[y][x] = symbols[y-1][x-1]; }
                    }
                    if symbols[y-1][x] != 1 {
                        // If adjacent pixel symbol:
                        //  If we have set pixel to symbol already: set similarity,
                        //  else set pixel to symbol
                        if symbols[y][x] != 0 {
                            same_symbols[symbols[y][x] as usize][symbols[y-1][x] as usize] = true;
                        } 
                        else { symbols[y][x] = symbols[y-1][x]; }
                    }
                    if symbols[y-1][x+1] != 1 {
                        // If adjacent pixel symbol:
                        //  If we have set pixel to symbol already: set similarity,
                        //  else set pixel to symbol
                        if symbols[y][x] != 0 {
                            same_symbols[symbols[y][x] as usize][symbols[y-1][x+1] as usize] = true;
                        } 
                        else { symbols[y][x] = symbols[y-1][x+1]; }
                    }
                    // If we haven't assigned to symbol: set new symbol
                    if symbols[y][x] == 0 {
                        symbols[y][x] = symbol_count as u32;
                        symbol_count += 1u32;
                    }
                }
            }
        }
        symbol_count -= 2;
        println!("{} : Initial symbols set",time(start));
        return symbol_count;
    }
    fn get_links(same_symbols:&Vec<Vec<bool>>) -> Vec<(usize,usize)> {
        let start = Instant::now();
        let mut links:Vec<(usize,usize)> = Vec::new();
        for i in 0..same_symbols.len() {
            for t in 0..same_symbols[i].len() { // same_symbols[a].len() === same_symbols[b].len()
                if i != t && same_symbols[i][t] {
                    links.push((i,t))
                }
            }
        }
        println!("{} : Got links",time(start));
        return links;
    }
    // Gets vector of which symbols to change. Index=current symbol, value=symbol to change to.
    fn symbol_changes(symbol_count:u32,links:&mut Vec<(usize,usize)>) -> Vec<u32> {
        let start = Instant::now();
        // Sets symbol number to unify pixels under
        // Sets all connected numbers to same number
        let mut change_symbols:Vec<u32> =  (2u32..symbol_count+2u32).collect();
        for i in 0..links.len() {
            // Gets lowest symbol in connection
            let connection = links[i];
            let (low,large) = if connection.0 < connection.1 { (connection.0,connection.1) } 
            else { (connection.1,connection.0) };
    
            // Adjusts symbols list
            change_symbols[large-2] = low as u32;
            for val in &mut change_symbols {
                if *val == large as u32 { 
                    *val = low as u32;
                }
            }
    
            // Adjusts connections list
            links[i] = (low,low);
            for link in links.iter_mut() {
                if link.0 == large { link.0 = low; }
                if link.1 == large { link.1 = low; }
            }
        }
        println!("{} : Set symbol changes",time(start));
        return change_symbols;
    }
    fn make_symbols_consectutive(symbol_count:u32,change_symbols:&Vec<u32>) -> (Vec<u32>,u32) {
        let start = Instant::now();
        // All symbols numbers are now covered by some range 2..x, rather than a series of numbers >=2
        let mut symbol_to:Vec<u32> = (2u32..symbol_count+2u32).collect();
        let mut consecutive_symbols:Vec<u32> = change_symbols.clone();
        let mut max_so_far:u32 = consecutive_symbols[0]; // This might always be 2u8, if so, maybe change this to that
        let mut counter:u32 = 2u32;
        for i in 1..consecutive_symbols.len() {
            if consecutive_symbols[i] > max_so_far {
                counter += 1;
                max_so_far = consecutive_symbols[i];
                symbol_to[consecutive_symbols[i] as usize - 2usize] = counter;
                consecutive_symbols[i] = counter;
            }
            else {
                consecutive_symbols[i] = symbol_to[consecutive_symbols[i] as usize - 2];
            }
        }
        println!("{} : Made symbol changes consecutive",time(start));
        return (consecutive_symbols,counter);
    }
    fn unify_symbols(width:usize,height:usize,counter:u32,symbols:&mut Vec<Vec<u32>>,change_symbols:&Vec<u32>,consecutive_symbols:&Vec<u32>) -> Vec<Vec<(usize,usize)>> {
        let start = Instant::now();
        let mut pixels_in_symbols:Vec<Vec<(usize,usize)>> = vec!(Vec::new();(counter-1) as usize);
        // O(n)
        // Unifies symbols and sets lists of pixels in each symbol
        for y in 0..height {
            for x in 0..width {
                if symbols[x][y] != 1 {
                    let symbol_num = symbols[x][y] as usize - 2usize;
                    let new_symbol_num = consecutive_symbols[change_symbols[symbol_num] as usize - 2usize];
                    symbols[x][y] = new_symbol_num;
                    pixels_in_symbols[new_symbol_num as usize - 2usize].push((x,y));
                    
                }
            }
        }
        println!("{} : Unified symbols",time(start));
        return pixels_in_symbols;
    }
    fn set_borders(width:usize,height:usize,path:&Path,symbols:&Vec<Vec<u32>>,pixels_in_symbols:&Vec<Vec<(usize,usize)>>) -> Vec<((usize,usize),(usize,usize))> {
        let start = Instant::now();
        let mut borders:Vec<((usize,usize),(usize,usize))> = vec!(((0usize,0usize),(0usize,0usize));pixels_in_symbols.len());
    
        for (symbol,border_limits) in izip!(pixels_in_symbols,&mut borders) {
            //print!("{},",symbol.len());
            let (mut min_x,mut min_y) = symbol[0];
            let (mut max_x,mut max_y) = symbol[0];
            for pixel in symbol {
                if pixel.0 < min_x { min_x = pixel.0; }
                else if pixel.0 > max_x { max_x = pixel.0; }
    
                if pixel.1 < min_y { min_y = pixel.1; }
                else if pixel.1 > max_y { max_y = pixel.1; }
            }
            *border_limits = ((min_x,min_y),(max_x,max_y));
        }
    
        // TODO Maybe print borders, might be bit too much data, I dunno
    
        let mut outline_img = image::open(path).unwrap().into_rgb();
    
        // Copies image
        for (x,y,pixel) in outline_img.enumerate_pixels_mut() {
            let val = if symbols[x as usize][y as usize] == 1 { 255 } else { 0 };
            *pixel = image::Rgb([val,val,val]);
        }
    
        // Sets borders
        for i in 0..borders.len() {
    
            let (mut min_x,mut min_y) = borders[i].0;
            let (mut max_x,mut max_y) = borders[i].1;
    
            min_x = if B_SPACING > min_x { 0 } else { min_x - B_SPACING }; // min_x - border_spacing < 0
            min_y = if B_SPACING > min_y { 0 } else { min_y - B_SPACING }; // min_y - border_spacing < 0
            max_x = if max_x + B_SPACING >= width { width-1 } else { max_x + B_SPACING };
            max_y = if max_y + B_SPACING >= height { height-1 } else { max_y + B_SPACING };
            
            // Add spacing to borders
            borders[i] = ((min_x,min_y),(max_x,max_y));
    
            //println!("min,max:({},{}),({},{})",min_x,min_y,max_x,max_y);
    
            let border_pixel = image::Rgb([255,0,0]); // Pixel to use as border
            // Sets horizontal borders
            for i in min_x..max_x {
                *outline_img.get_pixel_mut(i as u32,min_y as u32) = border_pixel;
                *outline_img.get_pixel_mut(i as u32,max_y as u32) = border_pixel;
            }
            // Sets vertical borders
            for i in min_y..max_y {
                *outline_img.get_pixel_mut(min_x as u32,i as u32) = border_pixel;
                *outline_img.get_pixel_mut(max_x as u32,i as u32) = border_pixel;
            }
            // Sets bottom corner border
            *outline_img.get_pixel_mut(max_x as u32,max_y as u32) = border_pixel;
            
        }
    
        outline_img.save("borders.png").unwrap();
        println!("{} : Set borders",time(start));
        return borders;
    }
    fn create_symbol_images(width:usize,height:usize,symbols:&Vec<Vec<u32>>,borders:&Vec<((usize,usize),(usize,usize))>) -> Vec<ImageBuffer<Luma<u8>,Vec<u8>>> {
        let start = Instant::now();
        let sizes:Vec<(usize,usize)> = borders.iter().map(|lims| ((lims.1).0-(lims.0).0+1,(lims.1).1-(lims.0).1+1)).collect();
        // Default constructs to all black pixels, thus we set symbol pixels to white in following loop
        // TODO Look into constructing with default white pixels and drawing black pixels
        let mut symbol_images:Vec<ImageBuffer<Luma<u8>,Vec<u8>>> = sizes.iter().map(|size| ImageBuffer::<Luma<u8>,Vec<u8>>::new(size.0 as u32,size.1 as u32)).collect();
    
    
        // O(n)
        // Draws symbol images
        for y in 0..height {
            for x in 0.. width {
                if symbols[x][y] != 1 {
                    let symbol = symbols[x][y] as usize;
                    let offset:(usize,usize) = borders[symbol-2].0;
    
                    let pixel = symbol_images[symbol-2].get_pixel_mut((x-offset.0) as u32,(y-offset.1) as u32);
                    *pixel = image::Luma([255]);
                }
            }
        }
        println!("{} : Created symbol images",time(start));
        return symbol_images;
    }
}

// Converts img raw into binary image which it returns as a 2d vector.
fn to_bin2dvec(width:usize,height:usize,img_raw:&mut Vec<u8>) -> Vec<Vec<u32>> {
    // 2d vector of size of image, where each pixel will be labelled as to which symbol it belongs
    let start = Instant::now();
    let mut symbols:Vec<Vec<u32>> = vec!(vec!(1u32;width as usize);height as usize);
    println!("width * height = length : {} * {} = {}|{}k|{}m",width,height,img_raw.len(),img_raw.len()/1000,img_raw.len()/1000000);
    // Leave x=0 and y=0 borders as 1u8:alloc
    //  It is not that not doing so would cause an error,
    //  but rather that doing so has no affect.
    for y in 0..height {
        for x in 0..width {
            let luma = img_raw[y*width+x];
            img_raw[y*width+x] = if luma < LUMA_BOUNDARY { 0 } else { 255 };
            symbols[y][x] = (img_raw[y*width+x] / 255) as u32;
        }
    }
    println!("{} : Converted image to binary",time(start));
    return symbols;
}

fn time(instant:Instant) -> String {
    let mut millis = instant.elapsed().as_millis();
    let seconds = (millis as f32 / 100f32).floor();
    millis = millis % 100;
    let time = format!("{:#02}:{:#02}",seconds,millis);
    return time;
}

// Nicely prints Vec<Vec<u8>> as matrix
#[allow(dead_code,non_snake_case)]
pub fn symbols_intial_prt(matrix:&Vec<Vec<u32>>) -> () {

    println!();
    let shape = (matrix.len(),matrix[0].len()); // shape[0],shape[1]=row,column
    let spacing = 1*shape.0;
    horizontal_number_line(shape.0);

    println!("    ┌{:─<1$}┐","",spacing);
    for row in 0..shape.1 {
        vertical_number_line(row);

        print!("│");
        for col in 0..shape.0 {
            if matrix[col][row] == 1 { print!("{}",matrix[col][row]);/*print!("{}",WHITE_SPACE_SYMBOL);*/ }
            else { print!("{}",matrix[col][row]); }
            
            
        }
        println!("│");
    }
    println!("    └{:─<1$}┘","",spacing);
    print!("   {:<1$}","",(spacing/2)-1);
    println!("   [{},{}]",shape.0,shape.1);
    println!();

    fn horizontal_number_line(rows:usize) -> () {
        print!("\n   ");
        for col in 0..rows/10 {
            print!("{: <1$}","",4);
            print!("{: >2}",col);
            print!("{: <1$}","",4);
        }
        print!("\n    ");
        for _ in 0..rows/10 {
            print!("┌{:─<1$}┐","",8);
        }
        print!("┌{:─<1$}","",rows%10);
        print!("\n    ");
        for col in 0..rows {
            print!("{: >1}",col%10)
        }
        println!();
    }

    fn vertical_number_line(row:usize) -> () {
        if row % 10 == 5 {
            print!("{: >2}",row/ 10);
        } else { print!("  "); }

        if row % 10 == 0 {
            print!("┌");
        }
        else if row % 10 == 9 {
            print!("└");
        }
        else {
            print!("│");
        }
        print!("{}",row % 10);
    }
}
#[allow(dead_code,non_snake_case)]
pub fn symbols_classified_prt(matrix:&Vec<Vec<u32>>) -> () {

    println!();
    let shape = (matrix.len(),matrix[0].len()); // shape[0],shape[1]=row,column
    let spacing = 2*shape.0;
    
    horizontal_number_line(shape.0);

    println!("    ┌─{:─<1$}┐","",spacing);
    for row in 0..shape.1 {
        vertical_number_line(row);

        print!("│");
        for col in 0..shape.0 {
            // TODO Do the whitespace print better
            if matrix[col][row] == 1 { print!(" {}",WHITE_SPACE_SYMBOL); }
            else { print!("{: >2}",matrix[col][row]); }
        }
        println!(" │");
    }
    println!("    └─{:─<1$}┘","",spacing);
    print!("{:<1$}","",(spacing/2)-1);
    println!("[{},{}]",shape.0,shape.1);
    println!();

    fn horizontal_number_line(rows:usize) -> () {
        print!("\n   ");
        for col in 0..rows/10 {
            print!("{: <1$}","",9);
            print!("{: >2}",col);
            print!("{: <1$}","",9);
        }
        print!("\n    ");
        for _ in 0..rows/10 {
            print!("┌{:─<1$}┐","",2*9);
        }
        print!("┌{:─<1$}","",rows%10);
        print!("\n    ");
        for col in 0..rows {
            print!("{: >2}",col%10)
        }
        println!();
    }

    fn vertical_number_line(row:usize) -> () {
        if row % 10 == 5 {
            print!("{: >2}",row/ 10);
        } else { print!("  "); }

        if row % 10 == 0 {
            print!("┌");
        }
        else if row % 10 == 9 {
            print!("└");
        }
        else {
            print!("│");
        }
        print!("{}",row % 10);
    }
}