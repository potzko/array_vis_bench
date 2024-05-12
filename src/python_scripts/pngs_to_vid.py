try:
    import cv2
except:
    import pip
    pip.main(['install', 'opencv-python'])
import os
cpu_count = os.cpu_count()
img_folder_path = '.\\\\\\tmp'
images = [img for img in os.listdir(img_folder_path) if img.endswith(".png")]

images.sort(key = lambda x: int(x[:-4]))

# desired video resolution
width, height = 2048 , 1080  # for example, to set the resolution to 640x480 pixels
size = (width, height)

frame_rate = 60 # frames per second
def vid(inp):
    num, images = inp[0], inp[1]
    out = cv2.VideoWriter(f'.\\tmp\\project{num}.mp4', cv2.VideoWriter_fourcc(*'mp4v'), frame_rate, size)

    for ind, image in enumerate(images):
        if ind % 100 == 0:
            print(image)
        img_path = os.path.join(img_folder_path, image)
        img = cv2.imread(img_path)
        
        # resizing the image to the desired resolution
        img_resized = cv2.resize(img, size)
        
        out.write(img_resized)

    out.release()
if __name__ == '__main__':
    def split_even_chuncks(arr, n):
        k, m = divmod(len(arr), n)
        print(f"splicing images to video, {n} chunks of size {len(arr) // n}")
        return [arr[i*k+min(i, m):(i+1)*k+min(i+1, m)] for i in range(n)]

    arrs = split_even_chuncks(images, cpu_count * 5)
    arrs = list(enumerate(arrs))

    from multiprocessing import Pool
    with Pool(cpu_count) as p:
        p.map(vid, arrs)

    def concatenate_videos(video_file_count):
        videos = [cv2.VideoCapture(f".\\tmp\\project{i}.mp4") for i in range(video_file_count)]
        print(videos)
        total_frames = 0


        # Create an output video writer
        fourcc = cv2.VideoWriter_fourcc(*'mp4v')
        out = cv2.VideoWriter('project.mp4', fourcc, frame_rate, size)

        for video in videos:
            ret, frame = video.read()
            while ret:
                out.write(frame)
                ret, frame = video.read()

        # Release resources
        for video in videos:
            video.release()
        out.release()


    # Example usage:
    concatenate_videos(len(arrs))