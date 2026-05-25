\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texnote}
\LoadClass[12pt]{article}
\RequirePackage{amsthm}
\RequirePackage{amssymb}
\RequirePackage{mathrsfs}
\RequirePackage{tikz-cd}

\RequirePackage{amsmath}

\RequirePackage{xr-hyper} 
%hyperlinks and referencing
\RequirePackage[hypertexnames=false]{hyperref} 
\RequirePackage[capitalize,nameinlink,noabbrev]{cleveref}


%setup external file links

\RequirePackage{import}
\ifx\HCode\UnDeFiNeD
    \subimport{../}{documents.tex}
\fi

\RequirePackage{slashed}

%setup geometry
\RequirePackage[margin=2.5cm]{geometry}
\RequirePackage{graphicx}
\graphicspath{{../../resources}}



\RequirePackage[style=apa]{biblatex}
\addbibresource{../../bibliography.bib}


\hypersetup{
    colorlinks = false,
    citecolor = cyan
}

\usepackage{../../template/style}

\theoremstyle{definitionstyle}
\newtheorem{axioma}{Axioma}
\newtheorem{postulado}{Postulado}
\newtheorem{definicion}{Definición}

\theoremstyle{examplestyle}
\newtheorem{ejemplo}{Ejemplo}
\newtheorem{ejemplos}[ejemplo]{Ejemplos}

\theoremstyle{exercisestyle}
\newtheorem{ejercicio}[ejemplo]{Ejercicio}

\theoremstyle{propositionstyle}
\newtheorem{proposicion}{Proposición}
\newtheorem{lema}[proposicion]{Lema}
\newtheorem{teorema}[proposicion]{Teorema}
\newtheorem{corolario}[proposicion]{Corolario}

\theoremstyle{remarkstyle}
\newtheorem{observacion}[definicion]{Observación}

\renewcommand{\maketitle}{
    \onehalfspacing
    \setlength{\abovedisplayskip}{5pt}
    \setlength{\belowdisplayskip}{7pt}
    \noindent\@author \hfill \@date
    \begin{center}
        \Huge \textbf{\@title}
    \end{center}
    \noindent
}

\usepackage{ifthen}

%TeXNotes Commands
\makeatletter
\newcommand{\ztxhtmlhref}[3]{\href{#1.html\##2}{#3}}
\newcommand{\ztxmaybehyperlink}[2]{%
    \ifx\HCode\UnDeFiNeD
        #1%
    \else
        #2%
    \fi
}
\newcommand{\exhyperref}[3][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ztxmaybehyperlink{%
                \ifx\HCode\UnDeFiNeD
                    \hyperref[#2-#1]{#3}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{#3}%
                \fi
            }{#3}%
        }%
        {% Referencia local (mismo comportamiento)
            \ztxmaybehyperlink{\hyperref[#1]{#3}}{#3}%
        }%
}

\newcommand{\excref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\cref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\cref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Referencia local
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\cref{#1}}
                    {\cref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\cref{#1}}{\cref{#1}}}%
        }%
}

\newcommand{\exref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\ref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\ref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Referencia local
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\ref{#1}}
                    {\ref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\ref{#1}}{\ref{#1}}}%
        }%
}
\makeatother

\newcommand{\transclude}[2][note]{\ExecuteMetaData[#2.tex]{#1}}

\newcommand{\chapter}[1]{\textbf{\LARGE#1}}

\usepackage{catchfilebetweentags}

%currentdoc command for make4ht fix 
\makeatletter
\newcommand\currentdoc[1]{\edef\@currentlabel{#1}\label{#1}}
\ifx\HCode\UnDeFiNeD
% No HTML anchors in PDF mode.
\else
\let\ztxoriglabel\label
\newcommand{\ztxhtmlanchor}[1]{\HCode{<a id='}#1\HCode{'></a>}}
\renewcommand{\label}[1]{%
    \ztxoriglabel{#1}%
    \ztxhtmlanchor{#1}%
    \ztxhtmlanchor{\jobname-#1}%
}
\let\ztxorig@href\href
\renewcommand{\href}[2]{%
    \ztxmaybehyperlink{\ztxorig@href{#1}{#2}}{#2}%
}
\let\ztxorig@hyperref\hyperref
\renewcommand{\hyperref}[2][]{%
    \ztxmaybehyperlink{\ztxorig@hyperref[#1]{#2}}{#2}%
}
\fi
\makeatother

\ProcessOptions

