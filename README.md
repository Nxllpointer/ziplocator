# Ziplocator
Predict US zipcode locations using a neural network \
*Uni project*

![ziplocator ui](https://github.com/user-attachments/assets/40fdbe6d-4634-43a1-b858-7b44b56c756e)


## Crates
### ziplocator-data
- Download dataset
- Create [polars](https://docs.rs/polars/latest/polars/) dataframe
- Specific data queries

### ziplocator-nn
- Neural network using [burn](https://docs.rs/burn/latest/burn/)
- Training
- Inference

### ziplocator-ui
- User interface with [iced](https://docs.rs/iced/latest/iced/)
- Interactive map
  - [OpenStreetMap](https://www.openstreetmap.org)
  - Rendered by [galileo](https://docs.rs/galileo/latest/galileo/)
- NN layer debug menu

## Architecture overview
![ziplocator-architecture-darkmode](https://github.com/user-attachments/assets/e2952042-e94c-44d5-8f5a-ddbbe67b1dd1#gh-dark-mode-only)
![ziplocator-architecture-lightmode](https://github.com/user-attachments/assets/5abb209a-f6e8-4dfc-8415-e6eb2287e75a#gh-light-mode-only)
