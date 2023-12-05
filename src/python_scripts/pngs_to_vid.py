
import cv2
import os


# specify the path for your images
img_folder_path = '.\\\\\\tmp'
images = [img for img in os.listdir(img_folder_path) if img.endswith(".png")]

images.sort(key = lambda x: int(x[:-4]))

# desired video resolution
width, height = 2048 , 1080  # for example, to set the resolution to 640x480 pixels
size = (width, height)

frame_rate = 60 # frames per second

#fourcc = cv2.VideoWriter_fourcc(*'ap4h')
#out = cv2.VideoWriter('output.mov', fourcc, frame_rate, size)

out = cv2.VideoWriter('project.mp4', cv2.VideoWriter_fourcc(*'mp4v'), frame_rate, size)

for ind, image in enumerate(images):
    if ind % 100 == 0:
        print(image)
    img_path = os.path.join(img_folder_path, image)
    img = cv2.imread(img_path)
    
    # resizing the image to the desired resolution
    img_resized = cv2.resize(img, size)
    
    out.write(img_resized)

out.release()