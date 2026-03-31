import os
import re

moved_files = {
    'API.md': '02-guia-tecnica/api.md',
    'COMMANDS.md': '03-comandos/00-referencia.md',
    'CONFIG.md': '01-guia-usuario/configuracion.md',
    'EXPORT.md': '01-guia-usuario/exportacion.md',
    'FUZZY.md': '01-guia-usuario/fuzzy.md',
    'QUICKSTART.md': '01-guia-usuario/quickstart.md'
}

docs_dir = os.path.abspath('docs')

def get_rel_path(from_file, to_file_rel_root):
    from_dir = os.path.dirname(os.path.abspath(from_file))
    to_abs = os.path.join(docs_dir, to_file_rel_root)
    return os.path.relpath(to_abs, from_dir)

def do_replacements():
    for root, dirs, files in os.walk(docs_dir):
        for name in files:
            if not name.endswith('.md'): continue
            file_path = os.path.join(root, name)
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            def replacer(match):
                text = match.group(1)
                link = match.group(2)
                if link.startswith('http'): return match.group(0)
                link_no_hash = link.split('#')[0]
                hash_part = '#' + link.split('#')[1] if '#' in link else ''
                if not link_no_hash: return match.group(0) # internal hash
                
                from_dir = os.path.dirname(os.path.abspath(file_path))
                target_abs = os.path.normpath(os.path.join(from_dir, link_no_hash))
                
                if not os.path.exists(target_abs):
                    # Check if target was pointing to old location in root
                    for old_name, new_name in moved_files.items():
                        if target_abs == os.path.join(docs_dir, old_name):
                            new_rel = get_rel_path(file_path, new_name)
                            return f"[{text}]({new_rel}{hash_part})"
                    
                    # If this is a moved file, fix relative paths that were previously correct from docs root.
                    original_file_loc = os.path.join(docs_dir, os.path.basename(file_path))
                    theoretical_target = os.path.normpath(os.path.join(os.path.dirname(original_file_loc), link_no_hash))
                    if os.path.exists(theoretical_target):
                        new_rel = os.path.relpath(theoretical_target, from_dir)
                        return f"[{text}]({new_rel}{hash_part})"
                        
                return match.group(0)
                
            new_content = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', replacer, content)
            if content != new_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(new_content)

do_replacements()
