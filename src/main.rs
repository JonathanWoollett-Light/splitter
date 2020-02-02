extern crate image;
use std::path::Path;
use image::{Luma,Pixel,ImageBuffer,Rgb};
use std::u8::MAX;
fn main() {
    let path = Path::new("/home/jonathan/Projects/splitter/images/mess.png");

    let img = image::open(path).unwrap().to_luma();
    
    let dims = img.dimensions();
    let (width,height) = (dims.0 as usize,dims.1 as usize);
    
    let img_raw:Vec<u8> = img.clone().into_raw();
    
    // 2d vector of size of image, each pixel labelled as to which symbol it belongs
    let mut symbols:Vec<Vec<u8>> = vec!(vec!(1u8;height as usize);width as usize);
    println!("{}",img_raw.len());
    println!("{}",width*height);
    println!("({},{})",width,height);
    println!("({},{})",symbols.len(),symbols[0].len());
    println!("max:y*width+x:{}",((height-1)*width)+width-1);
    // Leave x=0 and y=0 borders as all 0
    for x in 1..width {
        for y in 1..height {
            symbols[x][y] = img_raw[y*width+x] / 255;
            
        }
    }
    //u8_vec_vec__prt(&symbols);

    
    let mut same_symbols:Vec<Vec<bool>> = vec!(vec!(false;50usize);50usize); // Presume no more than 100 symbols in image
    let mut symbol_count:u8 = 2u8;
    
    
    // Sets intial symbol pixels
    for y in 1..height {
        for x in 1..width {
            if symbols[x][y] == 0 {
                if symbols[x-1][y] != 1 {
                    symbols[x][y] = symbols[x-1][y];
                }
                if symbols[x-1][y-1] != 1 {
                    // If adjacent pixel symbol:
                    //  If we have set pixel to symbol already: set similarity,
                    //  else set pixel to symbol
                    if symbols[x][y] != 0 {
                        same_symbols[symbols[x][y] as usize][symbols[x-1][y-1] as usize] = true;
                    } 
                    else { symbols[x][y] = symbols[x-1][y-1]; }
                    
                }
                if symbols[x][y-1] != 1 {
                    // If adjacent pixel symbol:
                    //  If we have set pixel to symbol already: set similarity,
                    //  else set pixel to symbol
                    if symbols[x][y] != 0 {
                        same_symbols[symbols[x][y] as usize][symbols[x][y-1] as usize] = true;
                    } 
                    else { symbols[x][y] = symbols[x][y-1]; }
                }
                if symbols[x+1][y-1] != 1 {
                    // If adjacent pixel symbol:
                    //  If we have set pixel to symbol already: set similarity,
                    //  else set pixel to symbol
                    if symbols[x][y] != 0 {
                        same_symbols[symbols[x][y] as usize][symbols[x+1][y-1] as usize] = true;
                    } 
                    else { symbols[x][y] = symbols[x+1][y-1]; }
                }
                // If we haven't assigned to symbol: set new symbol
                if symbols[x][y] == 0 {
                    symbols[x][y] = symbol_count as u8;
                    symbol_count += 1u8;
                }
            }
        }
    }
    symbol_count -= 2;
    println!("symbol_count: {}",symbol_count);
    let mut symbols_filtered = get_same_symbols(&same_symbols);
    println!("symbols_filtered: {:.?}",symbols_filtered);

    // Sets symbol number to unify pixels under
    // Sets all connected numbers to same number
    let mut change_symbols:Vec<u8> =  (2u8..symbol_count+2u8).collect();
    println!("change_symbols: {:.?}",change_symbols);
    for i in 0..symbols_filtered.len() {
        // Gets lowest symbol in connection
        let connection = symbols_filtered[i];
        let (low,large) = if connection.0 < connection.1 { (connection.0,connection.1) } 
        else { (connection.1,connection.0) };

        // Adjusts symbols list
        change_symbols[large-2] = low as u8;
        for val in &mut change_symbols {
            if *val == large as u8 { *val = low as u8; }
        }

        // Adjusts connections list
        symbols_filtered[i] = (low,low);
        for link in &mut symbols_filtered{
            if link.0 == large { link.0 = low; }
            if link.1 == large { link.1 = low; }
        }
    }
    println!("symbols_filtered: {:.?}",symbols_filtered);
    println!("change_symbols: {:.?}",change_symbols);

    let mut pixels_in_symbols:Vec<Vec<(usize,usize)>> = vec!(Vec::new();symbol_count as usize);

    // Unifies symbols and sets lists of pixels in each symbol
    for y in 0..height {
        for x in 0.. width {
            if symbols[x][y] != 1 {
                symbols[x][y] = change_symbols[symbols[x][y] as usize - 2usize];
                pixels_in_symbols[(symbols[x][y]-2u8) as usize].push((x,y));
            }
        }
    }

    //u8_vec_vec__prt(&symbols);

    // Removes empty lists (for symbols which where unified)
    pixels_in_symbols = pixels_in_symbols.into_iter().filter(|i| i.len()!=0).collect();
    //println!("pixels_in_symbols:\n{:.?}",pixels_in_symbols);

    let mut borders:Vec<((usize,usize),(usize,usize))> = vec!(((0usize,0usize),(0usize,0usize));pixels_in_symbols.len());
    
    for i in 0..pixels_in_symbols.len() {
        let (mut min_x,mut min_y) = pixels_in_symbols[i][0];
        let (mut max_x,mut max_y) = pixels_in_symbols[i][0];
        for t in 1..pixels_in_symbols[i].len() {
            if pixels_in_symbols[i][t].0 < min_x { min_x = pixels_in_symbols[i][t].0; }
            else if pixels_in_symbols[i][t].0 > max_x { max_x = pixels_in_symbols[i][t].0; }

            if pixels_in_symbols[i][t].1 < min_y { min_y = pixels_in_symbols[i][t].1; }
            else if pixels_in_symbols[i][t].1 > max_y { max_y = pixels_in_symbols[i][t].1; }
        }
        borders[i] = ((min_x,min_y),(max_x,max_y));
    }

    println!("boarders: {:.?}",borders);

    let mut outline_img = image::open(path).unwrap().into_rgb();
    //let mut outlint_img = ImageBuffer::<Rgb<u8>,Vec<u8>>::new(width as u32,height as u32);
    
    // Copies image
    for (x,y,p) in outline_img.enumerate_pixels_mut() {
        let val = if symbols[x as usize][y as usize] == 1 { 255 } else { 0 };
        *p = image::Rgb([val,val,val]);
    }
    // Sets borders
    for i in 0..borders.len() {
        let (mut min_x,mut min_y) = borders[i].0;
        let (mut max_x,mut max_y) = borders[i].1;
        
        let border_spacing = 2usize;
        min_x = if border_spacing > min_x { 0 } else { min_x - border_spacing }; // min_x - border_spacing < 0
        min_y = if border_spacing > min_y { 0 } else { min_y - border_spacing }; // min_y - border_spacing < 0
        max_x = if max_x + border_spacing >= width { width-1 } else { max_x + border_spacing };
        max_y = if max_y + border_spacing >= height { height-1 } else { max_y + border_spacing };
        
        //println!("min,max:({},{}),({},{})",min_x,min_y,max_x,max_y);

        for i in min_x..max_x {
            //println!("({},{}),({},{})",i,min_y,i,max_y);
            let pixel = outline_img.get_pixel_mut(i as u32,min_y as u32);
            *pixel = image::Rgb([255,0,0]);
            let pixel = outline_img.get_pixel_mut(i as u32,max_y as u32);
            *pixel = image::Rgb([255,0,0]);
            
        }
        for i in min_y..max_y {
            let pixel = outline_img.get_pixel_mut(min_x as u32,i as u32);
            *pixel = image::Rgb([255,0,0]);
            let pixel = outline_img.get_pixel_mut(max_x as u32,i as u32);
            *pixel = image::Rgb([255,0,0]);
        }
        // Sets bottom corner border
        let pixel = outline_img.get_pixel_mut(max_x as u32,max_y as u32);
        *pixel = image::Rgb([255,0,0]);
        
    }
    outline_img.save("borders.png").unwrap();
    
    
}

fn prt_u8_vec__as_2d((width,height):(usize,usize),vec:&Vec<u8>) -> () {
    println!();
    let shape = (width,height); // shape[0],shape[1]=row,column
    let spacing = 4*shape.0;
    println!("┌ {: <1$}┐","",spacing);
    for row in 0..shape.1 {
        print!("│ ");
        for col in 0..shape.0 {
            print!("{} ",vec[row*width+col]);
            
        }
        println!("│");
    }
    println!("└ {:<1$}┘","",spacing);
    print!("{:<1$}","",(spacing/2)-1);
    println!("[{},{}]",shape.0,shape.1);
    println!();
}

fn get_same_symbols(same_symbols:&Vec<Vec<bool>>) -> Vec<(usize,usize)> {
    print!("Same symbols: ");
    let mut same_symbols_return:Vec<(usize,usize)> = Vec::new();
    for i in 0..same_symbols.len() {
        for t in 0..same_symbols[i].len() { // same_symbols[a].len() === same_symbols[b].len()
            if i != t && same_symbols[i][t] {
                print!("({},{}),",i,t);
                same_symbols_return.push((i,t))
            }
        }
    }
    println!();
    return same_symbols_return;
}

// Nicely prints Vec<Vec<u8>> as matrix
pub fn u8_vec_vec__prt(matrix:&Vec<Vec<u8>>) -> () {

    println!();
    let shape = (matrix.len(),matrix[0].len()); // shape[0],shape[1]=row,column
    let spacing = 2*shape.0;
    println!("┌ {: <1$}┐","",spacing);
    for row in 0..shape.1 {
        print!("│ ");
        for col in 0..shape.0 {
            print!("{} ",matrix[col][row]);
            
        }
        println!("│");
    }
    println!("└ {:<1$}┘","",spacing);
    print!("{:<1$}","",(spacing/2)-1);
    println!("[{},{}]",shape.0,shape.1);
    println!();
}
