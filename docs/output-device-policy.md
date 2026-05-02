# Output Device Policy

Utagoe의 output device는 Settings > General의 device dropdown으로 선택한다.

## Policy

- 사용자가 dropdown에서 선택한 device에 고정한다.
- OS default device는 dropdown option 중 하나다.
- `Default Device`를 선택한 경우에만 OS default를 따른다.
- 특정 physical/logical device를 선택한 경우, OS default가 바뀌어도 해당 선택을 유지한다.

## Device Disappearance

선택한 device가 사라질 수 있다.

예:

- Bluetooth device disconnected
- USB DAC removed
- OS device id changed

Policy:

- 선택값 자체는 유지한다.
- 재생 가능한 output이 없으면 user-visible output error를 표시한다.
- 사용자가 `Default Device` 또는 다른 device로 바꿀 수 있어야 한다.

## Use Cases

1. 사용자는 Settings > General에서 `Default Device`를 선택하고 OS default output을 따른다.
2. 사용자는 특정 DAC를 선택하고 OS default 변경과 무관하게 해당 DAC를 사용한다.
3. 선택한 device가 사라지면 Utagoe는 선택값을 유지하되 output error를 표시한다.
