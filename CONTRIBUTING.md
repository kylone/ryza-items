# Ryza Items

This project will require collecting item data from the game Atelier Ryza. This data is collected in the form of YAML files, that will be used to create a database.

Here's the synthesis item 'Explosive Uni', as an example:

```yaml
---
Name: Explosive Uni
Item Number: 1
Level: 2
Classifications:
- Item Used
- Battle Item
- Attack
CC Cost: 2
Category:
- (Bomb)
Element:
- Fire: 1
Materials:
- (Uni)
- (Gunpowder)
- (Fuel)
Synthesis:
Required Materials: 2
Required Alchemy Level: 2
Material Loops:
    - Effect 1:
        Distance: 0
        Position: 0
        Material: (Uni)
        Levels:
        - Scatter: 
            Element:
            - Lightning: 3
        - Explosive:
            Element:
            - Lightning: 5
    - Effect 2:
        Distance: 1
        Position: 1
        Linked From Position: 0
        Material: (Gunpowder)
        Levels:
        - Uni Spike S:
            Element:
            - Fire: 1
        - Uni Spike M:
            Element:
            - Fire: 2
    - Effect 3:
        Distance: 1
        Position: 2
        Linked From Position: 0
        Material: (Fuel)
        Levels:
        - Surprise! S:
            Element:
            - Fire: 1
        - Surprise! M:
            Element:
            - Fire: 2
    - Traits:
        Distance: 2
        Position: 3
        Linked From Position: 1
        Material: (Fuel)
        Levels:
        - Trait Slot:
            Element:
            - Fire: 1
        - Trait Slot:
            Element:
            - Fire: 2
        - Trait Slot:
            Element:
            - Fire: 3
    - Effect 2:
        Distance: 2
        Position: 4
        Linked From Position: 1
        Material: (Gunpowder)
        Levels:
        - Uni Spike L: 
            Element:
            - Ice: 4
        Unlock:
        - Fire: 3
    - Recipe:
        Distance: 2
        Position: 5
        Linked From Position: 1
        Material: Red Supplement
        Levels:
        - Recipe Morph:
            Recipe: Craft
            Element:
            - Fire: 2
        Unlock:
        Fire: 2
    - Effect 3:
        Distance: 2
        Position: 6
        Linked From Position: 2
        Material: (Gunpowder)
        Levels:
        - Surprise! L:
            Element:
            - Wind: 2
    - Recipe:
        Distance: 2
        Position: 7
        Linked From Position: 2
        Material: Blue Supplement
        Levels:
        - Recipe Morph:
            Recipe: Ice Caltrop
            Element:
            - Ice: 2
        Unlock:
        - Wind: 1

```
