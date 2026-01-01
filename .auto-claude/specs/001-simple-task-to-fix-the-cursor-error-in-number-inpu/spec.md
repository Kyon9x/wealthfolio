# Quick Spec: Fix Number Input Cursor Position Bug

## Overview
Fix cursor jumping behavior in QuantityInput and MoneyInput components. When editing number fields in activity and bulk holding forms, new characters are appended to the end instead of inserting at the cursor position. This is a critical UX issue that makes number editing difficult and error-prone.

## Workflow Type
Bug Fix - Correct input behavior in existing form components

## Task Scope
This fix is scoped to two UI components in the packages/ui directory:
- QuantityInput component (`packages/ui/src/components/financial/quantity-input.tsx`)
- MoneyInput component (`packages/ui/src/components/financial/money-input.tsx`)

These components are used in:
- Activity add/edit forms (trade tab)
- Bulk holding forms (shares and average cost fields)

## Success Criteria
- [ ] Typing a number character inserts at cursor position, not at end
- [ ] Cursor position is preserved after value changes
- [ ] Works with negative numbers (QuantityInput)
- [ ] Works with comma-formatted numbers (MoneyInput - e.g., "1,234.56")
- [ ] Works with decimal numbers
- [ ] Compatible with react-hook-form controlled inputs
- [ ] Existing validation logic (decimal places, negatives) remains intact

## Task
Fix cursor jumping behavior in QuantityInput and MoneyInput components when editing numbers in activity and bulk holding forms.

## Files to Modify
- `packages/ui/src/components/financial/quantity-input.tsx` - Add cursor position preservation
- `packages/ui/src/components/financial/money-input.tsx` - Fix cursor position restoration

## Change Details

### Problem
When editing number fields, new characters are appended to the end instead of inserting at the cursor position. This happens because:

1. **QuantityInput**: Creates a synthetic event with processed value but doesn't preserve cursor position
2. **MoneyInput**: Attempts cursor restoration but it's lost when the component re-renders with new value from parent

### Solution
Both components need to:
1. Track cursor position before onChange
2. Use `useEffect` to restore cursor position after value changes
3. Use a ref to ensure cursor is restored after React updates the DOM

### Implementation

**QuantityInput** (`quantity-input.tsx`):
- Add `cursorPositionRef` to track cursor position
- Store cursor position in `handleChange` before calling onChange
- Add `useEffect` to restore cursor position when value changes

**MoneyInput** (`money-input.tsx`):
- The existing cursor position logic is incomplete
- Need to add `useEffect` to restore cursor after re-render
- Currently cursor restoration happens in handleChange but is lost when parent updates

## Verification
- [ ] Open activity form and navigate to trade tab
- [ ] Enter a number like "123.45"
- [ ] Place cursor between "2" and "3"
- [ ] Type "9" - it should insert as "1293.45" (cursor position preserved)
- [ ] Test in bulk holdings form for shares and average cost fields
- [ ] Test negative numbers in QuantityInput (if allowed)
- [ ] Test decimal edge cases

## Notes
- Both components are used in activity forms and bulk holdings forms
- The fix must work with react-hook-form's controlled inputs
- Must preserve existing validation (decimal places, negative sign handling)
- MoneyInput formats with commas (e.g., "1,234.56") which adds complexity
