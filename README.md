# MaybeUninit в Rust

## Что такое MaybeUninit

**`MaybeUninit<T>`** — обёртка, позволяющая **легально** работать с **неинициализированной** памятью. Раньше для этих целей использовали `mem::uninitialized`, но это давало **UB** для типов с **непустыми инвариантами** (например, `bool` или `NonNull`). `MaybeUninit` **обходит** это, потому что компилятор **знает**, что внутри может быть **мусор**, и **не делает** предположений.

## Проблема, которую решает

### `mem::uninitialized` — **опасно**

```rust
// ❌ UB — bool должен быть 0 или 1
let b: bool = unsafe { std::mem::uninitialized() };

// ❌ UB — NonNull не может быть null
let p: NonNull<i32> = unsafe { std::mem::uninitialized() };
```

**Причина:** компилятор **предполагает**, что `bool` — **валидный** (0 или 1), а `NonNull` — **не null**. Если память — **мусор**, это **нарушает** инварианты → **UB**.

### `MaybeUninit` — **безопасно**

```rust
// ✅ OK — MaybeUninit<bool> может содержать что угодно
let b: MaybeUninit<bool> = MaybeUninit::uninit();
```

**Причина:** `MaybeUninit<T>` **разрешает** **любые** байты — компилятор **знает** об этом.

## Разбор двух функций

### `make_array` — через `[u32; 4]`

```rust
fn make_array() -> [u32; 4] {
    let mb_uninit = MaybeUninit::<[u32; 4]>::uninit();

    let mut arr = unsafe {
        mb_uninit.assume_init()   // MaybeUninit<[u32; 4]> → [u32; 4]
    };

    for (ind, un) in arr.iter_mut().enumerate() {
        *un = ind as u32;         // запись значений
    }

    arr
}
```

- **`MaybeUninit::<[u32; 4]>::uninit()`** — **неинициализированный** массив `[u32; 4]`.
- **`.assume_init()`** — **меняет тип** на `[u32; 4]`.
- **Запись** через `*un = ...`.
- **Возврат** `[u32; 4]`.

**Работает**, потому что для `u32` **любые** байты **валидны**.

### `make_array2` — через `[MaybeUninit<u32>; 4]`

```rust
fn make_array2() -> [u32; 4] {
    let mut arr: [MaybeUninit<u32>; 4] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    for (ind, un) in arr.iter_mut().enumerate() {
        un.write(ind as u32);     // write вместо *un =
    }

    unsafe {
        std::mem::transmute(arr)  // [MaybeUninit<u32>; 4] → [u32; 4]
    }
}
```

- **`MaybeUninit::uninit().assume_init()`** — **массив** из 4 `MaybeUninit<u32>`.
- **`.write(v)`** — **безопасная** запись в `MaybeUninit<T>`.
- **`transmute`** — **переинтерпретирует** биты.

**Более корректный** вариант — **не** создаёт **невалидный** `[u32; 4]`.

## Классический паттерн

```rust
let mut arr: [MaybeUninit<u32>; 4] = unsafe {
    MaybeUninit::uninit().assume_init()
};

for (i, elem) in arr.iter_mut().enumerate() {
    elem.write(i as u32);
}

let arr: [u32; 4] = unsafe {
    std::mem::transmute(arr)
};

println!("{:?}", arr);   // [0, 1, 2, 3]
```

## Зачем нужен MaybeUninit

### 1. **Избежать** ненужной инициализации

```rust
// ❌ Обнуляет память
let arr: [u32; 1000] = [0; 1000];

// ✅ Не обнуляет
let arr: [MaybeUninit<u32>; 1000] = ...;
```

**Плюс:** **быстрее**, если значения **сразу** перезапишутся.

### 2. **FFI** — работа с C

```rust
extern "C" {
    fn malloc(size: usize) -> *mut u8;
}

let ptr = unsafe { malloc(1024) } as *mut MaybeUninit<u32>;
```

**C** **не обнуляет** память — `MaybeUninit` **соответствует**.

### 3. **Partial initialization** — частичная инициализация

```rust
let mut arr: [MaybeUninit<String>; 10] = ...;

for i in 0..5 {
    arr[i].write(format!("item {}", i));
}
// arr[5..10] — неинициализированы
```

**Полезно** для **больших** массивов или **ленивой** инициализации.

## Методы `MaybeUninit`

| Метод | Что делает | `unsafe`? |
|---|---|---|
| **`uninit()`** | Создаёт неинициализированный | ❌ |
| **`new(v)`** | Создаёт инициализированный | ❌ |
| **`write(v)`** | Записывает значение | ❌ |
| **`assume_init()`** | Приводит к `T` | ✅ |
| **`assume_init_ref()`** | `&T` | ✅ |
| **`assume_init_mut()`** | `&mut T` | ✅ |
| **`assume_init_read()`** | Читает `T` | ✅ |
| **`as_ptr()`** | `*const T` | ❌ |

## Сравнение `make_array` и `make_array2`

| | `make_array` | `make_array2` |
|---|---|---|
| **Тип** | `[u32; 4]` | `[MaybeUninit<u32>; 4]` |
| **`assume_init`** | На `MaybeUninit<[u32; 4]>` | На `MaybeUninit<[...]>` |
| **Запись** | `*un = ...` | `un.write(...)` |
| **Финал** | Прямой | `transmute` |
| **Безопасность** | ⚠️ Менее | ✅ Более |

**`make_array2`** — **лучше**, потому что **не создаёт** **невалидный** `[u32; 4]`.

## Сводная таблица

| Аспект | Описание |
|---|---|
| **`MaybeUninit<T>`** | Обёртка для неинициализированной памяти |
| **`uninit()`** | Создаёт неинициализированный |
| **`write(v)`** | Безопасная запись |
| **`assume_init()`** | Unsafe приведение к `T` |
| **Цели** | Производительность, FFI, partial init |

## Итог

- **`MaybeUninit<T>`** — **легальная** работа с **неинициализированной** памятью.
- **Заменяет** `mem::uninitialized` (который давал **UB**).
- **Цели:**
  - **избежать** ненужной инициализации;
  - **FFI** с C;
  - **частичная** инициализация.
- **`assume_init()`** — **меняет тип**, **не инициализирует**.
- **`write(v)`** — **безопасная** запись.
- **`transmute`** — **переинтерпретация** битов.
- **`make_array2`** — **лучше** `make_array` (безопаснее).
- **Опасно:** чтение без **записи** → **мусор** / **UB** (для `bool`, `char`, `&T`).
- **В вашем примере:** оба варианта дают `[0, 1, 2, 3]`, но **второй** — **корректнее**.
- **Правило:** `MaybeUninit` — **инструмент** для **unsafe-кода**; используйте **только** когда **нужно** обойти **инициализацию**.
