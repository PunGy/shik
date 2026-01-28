use crate::{
    count_args, define_help, define_native,
    eval::{
        error::RuntimeError,
        evaluator::Interpretator,
        native_functions::native_result,
        value::{
            EnvRef, ListRepr, NativeClosure, NativeContext, NativeFn, Value, ValueRef, ValueType,
        },
        EvalResult,
    },
    native_op,
};
use std::collections::HashMap;
use std::rc::Rc;

// ============================================================================
// Object Access Functions
// ============================================================================

// Get a value from an object by key
// Usage: object.get "key" obj
native_op!(ObjectGet, "object.get", [key, obj], {
    let obj = obj.expect_obj()?;
    let key = key.expect_string()?;

    match obj.get(key) {
        Some(v) => Ok(Rc::clone(v)),
        None => native_result(Value::Null),
    }
});

// Check if object has a key
// Usage: object.has "key" obj
native_op!(ObjectHas, "object.has", [key, obj], {
    let obj = obj.expect_obj()?;
    let key = key.expect_string()?;

    native_result(Value::Bool(obj.contains_key(key)))
});

// Get all keys from an object as a list
// Usage: object.keys obj
native_op!(ObjectKeys, "object.keys", [obj], {
    let obj = obj.expect_obj()?;

    let keys: Vec<ValueRef> = obj
        .keys()
        .map(|k| Rc::new(Value::String(k.clone())))
        .collect();

    native_result(Value::List(ListRepr::from_vec(keys)))
});

// Get all values from an object as a list
// Usage: object.values obj
native_op!(ObjectValues, "object.values", [obj], {
    let obj = obj.expect_obj()?;

    let values: Vec<ValueRef> = obj.values().map(|v| Rc::clone(v)).collect();

    native_result(Value::List(ListRepr::from_vec(values)))
});

// Get all entries from an object as a list of [key, value] pairs
// Usage: object.entries obj
native_op!(ObjectEntries, "object.entries", [obj], {
    let obj = obj.expect_obj()?;

    let entries: Vec<ValueRef> = obj
        .iter()
        .map(|(k, v)| {
            let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
            Rc::new(Value::List(ListRepr::from_vec(pair)))
        })
        .collect();

    native_result(Value::List(ListRepr::from_vec(entries)))
});

// Get the number of key-value pairs in an object
// Usage: object.len obj
native_op!(ObjectLen, "object.len", [obj], {
    let obj = obj.expect_obj()?;
    native_result(Value::Number(obj.len() as f64))
});

// Check if object is empty
// Usage: object.empty? obj
native_op!(ObjectIsEmpty, "object.empty?", [obj], {
    let obj = obj.expect_obj()?;
    native_result(Value::Bool(obj.is_empty()))
});

// ============================================================================
// Object Mutation Functions
// ============================================================================

// Set a value in an object by key (mutates object)
// Usage: object.set "key" obj value
native_op!(ObjectSet, "object.set", [key, obj, value], {
    let key = key.expect_string()?.clone();

    // Get mutable access to the object through the Value
    let obj_ptr = Rc::as_ptr(obj) as *mut Value;
    // SAFETY: Single-threaded interpreter, we're the only accessor
    unsafe {
        match &mut *obj_ptr {
            Value::Object(map) => {
                map.insert(key, Rc::clone(value));
                Ok(Rc::clone(value))
            }
            _ => Err(RuntimeError::mismatched_types(obj.get_type(), ValueType::Object)),
        }
    }
});

// Remove a key from an object (mutates object)
// Usage: object.remove "key" obj
native_op!(ObjectRemove, "object.remove", [key, obj], {
    let key = key.expect_string()?;

    // Get mutable access to the object through the Value
    let obj_ptr = Rc::as_ptr(obj) as *mut Value;
    // SAFETY: Single-threaded interpreter, we're the only accessor
    unsafe {
        match &mut *obj_ptr {
            Value::Object(map) => match map.remove(key) {
                Some(v) => Ok(v),
                None => native_result(Value::Null),
            },
            _ => Err(RuntimeError::mismatched_types(obj.get_type(), ValueType::Object)),
        }
    }
});

// ============================================================================
// Object Creation Functions
// ============================================================================

// Create an object from a list of [key, value] pairs
// Usage: object.from-entries [["a" 1] ["b" 2]]
native_op!(ObjectFromEntries, "object.from-entries", [entries], {
    let entries = entries.expect_list()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(entries.len());

    for entry in entries.iter() {
        let pair = entry.expect_list()?;
        let key = pair.get(0).ok_or(RuntimeError::invalid_application("(object.from-entries) must have at least two elements in order to make an object".to_string()))?;
        let key = key.expect_string()?.clone();
        let value = pair.get(1).ok_or(RuntimeError::invalid_application("(object.from-entries) must have at least two elements in order to make an object".to_string()))?;
        result.insert(key, value);
    }

    native_result(Value::Object(result))
});

// Merge two objects into a new object (second object's values override first)
// Usage: object.merge obj1 obj2
native_op!(ObjectMerge, "object.merge", [obj1, obj2], {
    let obj1 = obj1.expect_obj()?;
    let obj2 = obj2.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj1.len() + obj2.len());

    // Copy all from obj1
    for (k, v) in obj1.iter() {
        result.insert(k.clone(), Rc::clone(v));
    }

    // Override/add from obj2
    for (k, v) in obj2.iter() {
        result.insert(k.clone(), Rc::clone(v));
    }

    native_result(Value::Object(result))
});

// Create a shallow copy of an object
// Usage: object.clone obj
native_op!(ObjectClone, "object.clone", [obj], {
    let obj = obj.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj.len());

    for (k, v) in obj.iter() {
        result.insert(k.clone(), Rc::clone(v));
    }

    native_result(Value::Object(result))
});

// Create an object with only specified keys
// Usage: object.pick ["a" "b"] obj
native_op!(ObjectPick, "object.pick", [keys, obj], {
    let keys = keys.expect_list()?;
    let obj = obj.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(keys.len());

    for key_val in keys.iter() {
        let key = key_val.expect_string()?;
        if let Some(v) = obj.get(key) {
            result.insert(key.clone(), Rc::clone(v));
        }
    }

    native_result(Value::Object(result))
});

// Create an object without specified keys
// Usage: object.omit ["a" "b"] obj
native_op!(ObjectOmit, "object.omit", [keys, obj], {
    let keys = keys.expect_list()?;
    let obj = obj.expect_obj()?;

    // Collect keys to omit into a set for O(1) lookup
    let omit_keys: std::collections::HashSet<String> = keys
        .iter()
        .filter_map(|k| k.expect_string().ok().map(|s| s.clone()))
        .collect();

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj.len());

    for (k, v) in obj.iter() {
        if !omit_keys.contains(k) {
            result.insert(k.clone(), Rc::clone(v));
        }
    }

    native_result(Value::Object(result))
});

// ============================================================================
// Higher-Order Object Functions
// ============================================================================

// Iterate over object entries, calling function with [key, value] for each
// Usage: object.iterate callback obj
native_op!(ObjectIterate, "object.iterate", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        ctx.apply(func, &entry, ctx.env)?;
    }

    native_result(Value::Null)
});

// Map over object values, creating a new object with transformed values
// Usage: object.map mapper obj
native_op!(ObjectMap, "object.map", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj.len());

    for (k, v) in obj.iter() {
        let mapped = ctx.apply(func, v, ctx.env)?;
        result.insert(k.clone(), mapped);
    }

    native_result(Value::Object(result))
});

// Map over object entries with access to both key and value
// Callback receives [key, value] and should return new value
// Usage: object.map-entries mapper obj
native_op!(ObjectMapEntries, "object.map-entries", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj.len());

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        let mapped = ctx.apply(func, &entry, ctx.env)?;
        result.insert(k.clone(), mapped);
    }

    native_result(Value::Object(result))
});

// Filter object entries, keeping only those where predicate returns true
// Predicate receives [key, value]
// Usage: object.filter predicate obj
native_op!(ObjectFilter, "object.filter", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    let mut result: HashMap<String, ValueRef> = HashMap::with_capacity(obj.len());

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        let keep = ctx.apply(func, &entry, ctx.env)?.expect_bool()?;
        if keep {
            result.insert(k.clone(), Rc::clone(v));
        }
    }

    native_result(Value::Object(result))
});

// Fold over object entries, reducing to a single value
// Reducer receives accumulator and [key, value]
// Usage: object.fold initial reducer obj
native_op!(ObjectFold, "object.fold", [init, func, obj], ctx, {
    let obj = obj.expect_obj()?;

    let mut acc = Rc::clone(init);

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        // Apply function to accumulator first, then to entry (curried)
        let partial = ctx.apply(func, &acc, ctx.env)?;
        acc = ctx.apply(&partial, &entry, ctx.env)?;
    }

    Ok(acc)
});

// Check if any entry satisfies predicate
// Predicate receives [key, value]
// Usage: object.any predicate obj
native_op!(ObjectAny, "object.any", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        if ctx.apply(func, &entry, ctx.env)?.expect_bool()? {
            return native_result(Value::Bool(true));
        }
    }

    native_result(Value::Bool(false))
});

// Check if all entries satisfy predicate
// Predicate receives [key, value]
// Usage: object.all predicate obj
native_op!(ObjectAll, "object.all", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        if !ctx.apply(func, &entry, ctx.env)?.expect_bool()? {
            return native_result(Value::Bool(false));
        }
    }

    native_result(Value::Bool(true))
});

// Find first entry satisfying predicate, returns [key, value] or null
// Predicate receives [key, value]
// Usage: object.find predicate obj
native_op!(ObjectFind, "object.find", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        if ctx.apply(func, &entry, ctx.env)?.expect_bool()? {
            return Ok(entry);
        }
    }

    native_result(Value::Null)
});

// Find key of first entry satisfying predicate, returns key or null
// Predicate receives [key, value]
// Usage: object.find-key predicate obj
native_op!(ObjectFindKey, "object.find-key", [func, obj], ctx, {
    let obj = obj.expect_obj()?;

    for (k, v) in obj.iter() {
        let pair = vec![Rc::new(Value::String(k.clone())), Rc::clone(v)];
        let entry = Rc::new(Value::List(ListRepr::from_vec(pair)));
        if ctx.apply(func, &entry, ctx.env)?.expect_bool()? {
            return native_result(Value::String(k.clone()));
        }
    }

    native_result(Value::Null)
});

// ============================================================================
// Module Binding
// ============================================================================

pub fn bind_object_module(env: &EnvRef, inter: Rc<Interpretator>) {
    // Module help
    env.define_help(
        "object.".to_string(),
        "object module:

Access:
- object.get: gets value by key
- object.has: checks if key exists
- object.keys: returns list of keys
- object.values: returns list of values
- object.entries: returns list of [key, value] pairs
- object.len: returns number of entries
- object.empty?: checks if empty

Mutation:
- object.set: sets value by key (mutates object)
- object.remove: removes key (mutates object)

Creation:
- object.from-entries: creates object from [[key value] ...] list
- object.merge: merges two objects into new object
- object.clone: creates shallow copy
- object.pick: creates object with only specified keys
- object.omit: creates object without specified keys

Higher-order:
- object.iterate: iterates over entries
- object.map: transforms values
- object.map-entries: transforms values with access to keys
- object.filter: keeps entries matching predicate
- object.fold: reduces to single value
- object.any: true if any entry matches
- object.all: true if all entries match
- object.find: finds first matching entry
- object.find-key: finds key of first matching entry"
            .to_string(),
    );

    // Access functions
    define_native!(ObjectGet, env, inter);
    define_help!(
        ObjectGet,
        env,
        "[key:string object]: gets value by key, returns null if not found\n\nobject.get \"name\" {:name \"Alice\" :age 30}  ; \"Alice\""
    );

    define_native!(ObjectHas, env, inter);
    define_help!(
        ObjectHas,
        env,
        "[key:string object]: checks if object has key\n\nobject.has \"name\" {:name \"Alice\"}  ; true"
    );

    define_native!(ObjectKeys, env, inter);
    define_help!(
        ObjectKeys,
        env,
        "[object]: returns list of all keys\n\nobject.keys {:a 1 :b 2}  ; [\"a\" \"b\"]"
    );

    define_native!(ObjectValues, env, inter);
    define_help!(
        ObjectValues,
        env,
        "[object]: returns list of all values\n\nobject.values {:a 1 :b 2}  ; [1 2]"
    );

    define_native!(ObjectEntries, env, inter);
    define_help!(
        ObjectEntries,
        env,
        "[object]: returns list of [key, value] pairs\n\nobject.entries {:a 1 :b 2}  ; [[\"a\" 1] [\"b\" 2]]"
    );

    define_native!(ObjectLen, env, inter);
    define_help!(
        ObjectLen,
        env,
        "[object]: returns number of key-value pairs\n\nobject.len {:a 1 :b 2}  ; 2"
    );

    define_native!(ObjectIsEmpty, env, inter);
    define_help!(
        ObjectIsEmpty,
        env,
        "[object]: returns true if object has no entries\n\nobject.empty? {}  ; true"
    );

    // Mutation functions
    define_native!(ObjectSet, env, inter);
    define_help!(
        ObjectSet,
        env,
        "[key:string object value]: sets value by key (mutates object)\n\nobject.set \"name\" myobj \"Bob\""
    );

    define_native!(ObjectRemove, env, inter);
    define_help!(
        ObjectRemove,
        env,
        "[key:string object]: removes key from object (mutates), returns removed value or null\n\nobject.remove \"name\" myobj"
    );

    // Creation functions
    define_native!(ObjectFromEntries, env, inter);
    define_help!(
        ObjectFromEntries,
        env,
        "[entries:list]: creates object from list of [key, value] pairs\n\nobject.from-entries [[\"a\" 1] [\"b\" 2]]  ; {:a 1 :b 2}"
    );

    define_native!(ObjectMerge, env, inter);
    define_help!(
        ObjectMerge,
        env,
        "[object object]: merges two objects, second overrides first\n\nobject.merge {:a 1 :b 2} {:b 3 :c 4}  ; {:a 1 :b 3 :c 4}"
    );

    define_native!(ObjectClone, env, inter);
    define_help!(
        ObjectClone,
        env,
        "[object]: creates shallow copy of object\n\nobject.clone {:a 1 :b 2}"
    );

    define_native!(ObjectPick, env, inter);
    define_help!(
        ObjectPick,
        env,
        "[keys:list object]: creates object with only specified keys\n\nobject.pick [\"a\" \"c\"] {:a 1 :b 2 :c 3}  ; {:a 1 :c 3}"
    );

    define_native!(ObjectOmit, env, inter);
    define_help!(
        ObjectOmit,
        env,
        "[keys:list object]: creates object without specified keys\n\nobject.omit [\"b\"] {:a 1 :b 2 :c 3}  ; {:a 1 :c 3}"
    );

    // Higher-order functions
    define_native!(ObjectIterate, env, inter);
    define_help!(
        ObjectIterate,
        env,
        "[callback:lambda object]: calls function with [key, value] for each entry\n\nobject.iterate (fn [[k v]] print \"{k}: {v}\") {:a 1 :b 2}"
    );

    define_native!(ObjectMap, env, inter);
    define_help!(
        ObjectMap,
        env,
        "[mapper:lambda object]: transforms values, returns new object\n\nobject.map (fn [v] (* v 2)) {:a 1 :b 2}  ; {:a 2 :b 4}"
    );

    define_native!(ObjectMapEntries, env, inter);
    define_help!(
        ObjectMapEntries,
        env,
        "[mapper:lambda object]: transforms values with access to keys, mapper receives [key, value]\n\nobject.map-entries (fn [[k v]] + k (string v)) {:a 1 :b 2}  ; {:a \"a1\" :b \"b2\"}"
    );

    define_native!(ObjectFilter, env, inter);
    define_help!(
        ObjectFilter,
        env,
        "[predicate:lambda object]: keeps entries where predicate returns true, predicate receives [key, value]\n\nobject.filter (fn [[k v]] (> v 1)) {:a 1 :b 2 :c 3}  ; {:b 2 :c 3}"
    );

    define_native!(ObjectFold, env, inter);
    define_help!(
        ObjectFold,
        env,
        "[initial:value reducer:lambda object]: reduces object to single value, reducer receives acc and [key, value]\n\nobject.fold 0 (fn [acc [k v]] (+ acc v)) {:a 1 :b 2}  ; 3"
    );

    define_native!(ObjectAny, env, inter);
    define_help!(
        ObjectAny,
        env,
        "[predicate:lambda object]: returns true if any entry satisfies predicate\n\nobject.any (fn [[k v]] (> v 5)) {:a 1 :b 10}  ; true"
    );

    define_native!(ObjectAll, env, inter);
    define_help!(
        ObjectAll,
        env,
        "[predicate:lambda object]: returns true if all entries satisfy predicate\n\nobject.all (fn [[k v]] (> v 0)) {:a 1 :b 2}  ; true"
    );

    define_native!(ObjectFind, env, inter);
    define_help!(
        ObjectFind,
        env,
        "[predicate:lambda object]: finds first entry satisfying predicate, returns [key, value] or null\n\nobject.find (fn [[k v]] (> v 1)) {:a 1 :b 2}  ; [\"b\" 2]"
    );

    define_native!(ObjectFindKey, env, inter);
    define_help!(
        ObjectFindKey,
        env,
        "[predicate:lambda object]: finds key of first entry satisfying predicate, returns key or null\n\nobject.find-key (fn [[k v]] (> v 1)) {:a 1 :b 2}  ; \"b\""
    );
}
