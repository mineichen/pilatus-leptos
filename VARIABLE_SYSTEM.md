# Variable System Documentation

## Overview

The variable system allows UI components to reference values from variables instead of storing local copies. This is useful for configuration systems where values can be overridden at different levels (e.g., device-specific vs. system-wide defaults).

## Core Components

### 1. `PilatusPrimitiveValue<T>`

Located in `pilatus-leptos/src/impex_strategy.rs`

A wrapper around a value that can be either:
- **Value**: A concrete local value of type `T`
- **Variable**: A reference to a variable name (stored as a string up to 30 chars)

```rust
pub struct PilatusPrimitiveValue<T> {
    pub is_explicit: bool,
    pub value: ValueKind<T>,
}

pub enum ValueKind<T> {
    Variable(Variable),  // Variable reference
    Value(T),            // Concrete value
}
```

### 2. `LeafRwSignal<T>`

Located in `pilatus-leptos/src/leaf_rw_signal.rs`

A reactive signal wrapper around `Signal<PilatusPrimitiveValue<T>>` that provides convenient methods for working with variables:

**Key Methods:**
- `get_value()` - Gets the actual T value (panics if unresolved variable)
- `set_value(value: T)` - Sets as an explicit local value
- `is_variable()` - Checks if currently a variable reference
- `get_variable_name()` - Gets the variable name if it's a variable
- `convert_to_variable(name: &str)` - Converts current value to a variable reference
- `convert_to_local(value: T)` - Converts a variable back to a local value

### 3. `VariableInput` Component

Located in `pilatus-leptos/src/variable_input.rs`

A custom Thaw-based Input component that:
- Shows whether the value is from a variable or local
- Displays the variable name with a 🔗 icon when using a variable
- Allows converting local values to variables
- Allows converting variables back to local values
- Disables editing when using a variable (to prevent confusion)

**Usage:**
```rust
use pilatus_leptos::{LeafRwSignal, VariableInput};

#[component]
fn MyForm() -> impl IntoView {
    let my_value = LeafRwSignal::new("default value".to_string());
    
    view! {
        <VariableInput
            value=my_value
            label="My Setting"
        />
    }
}
```

## Serialization Format

When serialized to JSON:

**Local Value:**
```json
"Hello World"
```

**Variable Reference:**
```json
{
  "__var": "myVariableName"
}
```

## Example Workflow

1. **User edits a value normally:**
   - Input shows the current value
   - User can edit freely
   - Changes are saved as local values

2. **User clicks "🔗 Use Variable":**
   - Dialog appears asking for variable name
   - After entering name, the value becomes a variable reference
   - Input becomes disabled (shows variable name instead)
   - JSON is saved as `{"__var": "variableName"}`

3. **User clicks "Use Local Value":**
   - Variable reference is converted back to local value
   - Input becomes editable again
   - Current value is preserved

## Integration with DeviceContext

The `DeviceContext` (in `device_context.rs`) manages:
- Loading/saving device configurations
- Debouncing changes (250ms delay)
- Syncing changes to the server

When a `LeafRwSignal` value changes:
1. Change is captured in `DeviceContext`
2. After 250ms of no changes, it's sent to the server
3. External updates trigger notifications in the UI

## TODO / Future Improvements

1. **Variable Resolution**: Currently, trying to get the value of an unresolved variable panics. You'll need to implement variable resolution from a variable store/context.

2. **Variable Validation**: Add validation to ensure variable names are valid and exist.

3. **Variable Browser**: Create a UI component to browse available variables.

4. **Type Safety**: Consider adding type information to variables to prevent type mismatches.

5. **Bi-directional Sync**: The `From<LeafRwSignal<T>> for Model<T>` implementation needs work for proper two-way binding with standard Thaw components.

## Testing

Tests are located in `pilatus-leptos/src/impex_strategy.rs`:
- `deserialize_value` - Tests normal value deserialization
- `deserialize_value_with_variable_name` - Tests variable reference deserialization

Run tests with:
```bash
cargo test --lib impex_strategy
```

