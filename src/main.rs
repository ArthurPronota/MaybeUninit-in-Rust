use std::mem::MaybeUninit ;

fn make_array() ->[u32;4] {
    // Создаёт неиницализированный тип MaybeUninit<[u32; 4]>
    let mb_uninit = MaybeUninit::<[u32;4]>::uninit() ;

    let mut arr = unsafe {
        // Меняет тип MaybeUninit<[u32; 4]> в [u32; 4]
        mb_uninit.assume_init()
    } ;

    // установка значений
    for (ind, un) in arr.iter_mut().enumerate() {
        *un = ind as u32;
    }

    arr
}

fn make_array2() ->[u32;4] {

    let mut arr: [MaybeUninit<u32>; 4] = unsafe {
        // создаёт неинициализированный экземпляр MaybeUninit<[MaybeUninit<u32>; 4]>
        MaybeUninit::uninit()
            .assume_init()  // Преобразует MaybeUninit<[MaybeUninit<u32>; 4]> в [MaybeUninit<u32>; 4]
                            // Происходит смена типа.
    };

    for (ind, un) in arr.iter_mut().enumerate() {
        un
            .write(ind as u32) // установка значения
            ;
    } 

    unsafe {
        // Переинтерпретирует биты значения одного типа как биты значения 
        // другого типа: [MaybeUninit<u32>; 4] > [u32; 4]        
        std::mem::transmute(arr)
    }
}

fn main() {
    let arr1 = make_array() ;

    println!("{:?}", arr1) ;    // Out: [0, 1, 2, 3]

    let arr2 = make_array2() ;
    println!("{:?}", arr2) ;    // Out: [0, 1, 2, 3]
}
